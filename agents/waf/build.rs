use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if we're in standalone mode (no ModSecurity)
    if cfg!(feature = "standalone") {
        println!("cargo:warning=Building in standalone mode without ModSecurity");
        return Ok(());
    }

    // Determine which ModSecurity version to use
    let use_modsec3 = cfg!(feature = "modsecurity3");
    let use_modsec2 = cfg!(feature = "modsecurity2");

    if !use_modsec3 && !use_modsec2 {
        println!("cargo:warning=No ModSecurity version specified, defaulting to v3");
    }

    // Try to find ModSecurity using pkg-config
    let modsec_name = if use_modsec2 {
        "modsecurity"
    } else {
        "modsecurity" // v3 also uses "modsecurity" in pkg-config
    };

    let lib = match pkg_config::probe_library(modsec_name) {
        Ok(lib) => lib,
        Err(e) => {
            // Fallback to manual configuration
            println!(
                "cargo:warning=pkg-config failed: {}, trying manual configuration",
                e
            );

            // Check common installation paths
            let possible_paths = vec![
                "/usr/local/modsecurity",
                "/usr/local",
                "/opt/modsecurity",
                "/usr",
            ];

            let mut found_path = None;
            for path in &possible_paths {
                let inc_path = format!("{}/include", path);
                let lib_path = format!("{}/lib", path);

                if std::path::Path::new(&inc_path).exists()
                    && std::path::Path::new(&lib_path).exists()
                {
                    found_path = Some((inc_path, lib_path));
                    break;
                }
            }

            match found_path {
                Some((inc_path, lib_path)) => {
                    println!("cargo:rustc-link-search=native={}", lib_path);
                    println!("cargo:rustc-link-lib=modsecurity");
                    println!("cargo:include={}", inc_path);

                    pkg_config::Config {
                        atleast_version: None,
                        extra_args: vec![],
                        print_system_cflags: false,
                        print_system_libs: false,
                        cargo_metadata: true,
                        env_metadata: false,
                        statik: false,
                    }
                    .probe(modsec_name)?
                }
                None => {
                    eprintln!("ERROR: ModSecurity not found!");
                    eprintln!("Please install ModSecurity and ensure it's in your PKG_CONFIG_PATH");
                    eprintln!(
                        "Or build with --features standalone for testing without ModSecurity"
                    );
                    std::process::exit(1);
                }
            }
        }
    };

    // Generate bindings using bindgen
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate_comments(true)
        .generate_inline_functions(true)
        .allowlist_function("modsec_.*")
        .allowlist_function("msc_.*")
        .allowlist_type("ModSecurity.*")
        .allowlist_type("Transaction.*")
        .allowlist_type("RulesSet.*")
        .allowlist_type("ModSecurityIntervention.*")
        .allowlist_var("MODSEC_.*")
        .allowlist_var("MSC_.*")
        .derive_default(true)
        .derive_debug(true)
        .impl_debug(true);

    // Add include paths from pkg-config
    for include in &lib.include_paths {
        builder = builder.clang_arg(format!("-I{}", include.display()));
    }

    // Add version-specific defines
    if use_modsec3 {
        builder = builder.clang_arg("-DMODSECURITY_VERSION_NUM=030000");
    } else if use_modsec2 {
        builder = builder.clang_arg("-DMODSECURITY_VERSION_NUM=020900");
    }

    // Generate bindings
    let bindings = builder
        .generate()
        .expect("Unable to generate ModSecurity bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Link additional dependencies that ModSecurity might need
    println!("cargo:rustc-link-lib=pcre");
    println!("cargo:rustc-link-lib=xml2");
    println!("cargo:rustc-link-lib=curl");
    println!("cargo:rustc-link-lib=yajl");
    println!("cargo:rustc-link-lib=maxminddb");

    // On Linux, we might need these
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=lua5.1");
        println!("cargo:rustc-link-lib=z");
    }

    // Create a simple C wrapper for easier FFI
    cc::Build::new()
        .file("src/modsec_wrapper.c")
        .include("/usr/local/modsecurity/include")
        .compile("modsec_wrapper");

    Ok(())
}
