# Sentinel Documentation

This directory contains the source for Sentinel's documentation, built using [mdBook](https://github.com/rust-lang/mdBook).

## Prerequisites

The documentation build system is integrated with [mise](https://mise.jdx.dev/), Sentinel's task runner. All documentation tasks are available as mise tasks.

### Installing mdBook

To install mdBook and optional plugins:

```bash
mise run book-install
```

For optional plugins (mermaid diagrams, link checking, etc.):

```bash
INSTALL_PLUGINS=true mise run book-install
```

## Available Tasks

All documentation tasks are available through mise. To see all book-related tasks:

```bash
mise tasks | grep book
```

### Core Tasks

| Task | Description | Usage |
|------|-------------|-------|
| `book-install` | Install mdBook and optional plugins | `mise run book-install` |
| `book-build` | Build the documentation | `mise run book-build` |
| `book-serve` | Serve docs with live reload | `mise run book-serve` |
| `book-clean` | Clean build artifacts | `mise run book-clean` |
| `book-test` | Test for broken links and issues | `mise run book-test` |
| `book-watch` | Watch and rebuild on changes | `mise run book-watch` |
| `book-stats` | Show documentation statistics | `mise run book-stats` |
| `book-new-page` | Create a new documentation page | `mise run book-new-page PATH=guide/example.md` |
| `book-deploy` | Deploy to GitHub Pages | `mise run book-deploy` |

### Task Options

Many tasks support environment variables for customization:

#### book-serve
```bash
# Serve on custom port
PORT=8080 mise run book-serve

# Don't open browser automatically
OPEN_BROWSER=false mise run book-serve
```

#### book-new-page
```bash
# Create a new page with custom title and template
mise run book-new-page PATH=guides/docker.md TITLE="Docker Deployment" TEMPLATE=guide

# Available templates: basic, guide, reference, api
```

#### book-deploy
```bash
# Deploy with custom domain
CNAME=docs.example.com mise run book-deploy

# Force push to gh-pages
FORCE_PUSH=true mise run book-deploy

# Custom commit message
MESSAGE="Update API docs" mise run book-deploy
```

## Quick Start

1. **Install dependencies:**
   ```bash
   mise run book-install
   ```

2. **Serve the documentation locally:**
   ```bash
   mise run book-serve
   ```
   This will start a development server at `http://localhost:3000` with live reload.

3. **Build for production:**
   ```bash
   mise run book-build
   ```
   The built documentation will be in `book/book/`.

## Documentation Structure

```
book/
├── book.toml           # mdBook configuration
├── src/                # Documentation source files
│   ├── SUMMARY.md      # Table of contents
│   ├── introduction.md # Main introduction page
│   ├── getting-started/
│   ├── concepts/
│   ├── configuration/
│   ├── service-types/
│   ├── features/
│   ├── advanced/
│   ├── deployment/
│   ├── operations/
│   ├── reference/
│   ├── examples/
│   ├── development/
│   └── appendix/
└── book/               # Generated HTML (gitignored)
```

## Writing Documentation

### Creating New Pages

Use the `book-new-page` task to create new documentation:

```bash
# Create a basic page
mise run book-new-page PATH=guides/my-guide.md

# Create with specific template
mise run book-new-page PATH=reference/api/endpoints.md TEMPLATE=api

# Create with custom title
mise run book-new-page PATH=examples/nginx.md TITLE="Nginx Migration Guide"
```

The task will:
- Create the file with appropriate template
- Optionally add it to SUMMARY.md
- Provide next steps for editing

### Manual Page Creation

1. Create a new Markdown file in the appropriate directory under `src/`
2. Add an entry to `src/SUMMARY.md` to include it in the table of contents
3. Write your content using standard Markdown

### Style Guide

- Use clear, concise language
- Include code examples where appropriate
- Use proper heading hierarchy (# for title, ## for main sections, etc.)
- Add cross-references to related topics using relative links
- Include practical examples for complex concepts

### Code Blocks

Use language-specific code blocks for syntax highlighting:

````markdown
```rust
fn main() {
    println!("Hello, Sentinel!");
}
```
````

For configuration examples, use `kdl`:

````markdown
```kdl
route "api" {
    pattern "/api/*"
    service_type "api"
}
```
````

### Linking

- Internal links: Use relative paths like `[Installation](./getting-started/installation.md)`
- External links: Use full URLs like `[Pingora](https://github.com/cloudflare/pingora)`
- Anchor links: Use `[Section](#section-name)` for same-page navigation

## Testing

Test the documentation for broken links and other issues:

```bash
mise run book-test
```

This will:
- Run mdBook's built-in tests
- Check for broken internal links (if linkcheck plugin is installed)
- Display documentation statistics

## View Statistics

To see comprehensive documentation statistics:

```bash
mise run book-stats
```

This shows:
- File and content statistics
- Documentation structure breakdown
- Code example counts
- Section-specific metrics
- Completion status

## Deployment

### GitHub Pages

Deploy the documentation to GitHub Pages:

```bash
mise run book-deploy
```

With custom domain:

```bash
CNAME=docs.sentinel.io mise run book-deploy
```

The documentation will be available at:
- With CNAME: `https://docs.sentinel.io/`
- Without CNAME: `https://[owner].github.io/sentinel/`

### Manual Deployment

Build and deploy to any static hosting service:

```bash
mise run book-build
# Upload book/book/ directory to your hosting service
```

## Troubleshooting

### mdBook Not Found

If you get "mdBook is not installed" errors:

```bash
mise run book-install
```

### Port Already in Use

If port 3000 is already in use:

```bash
PORT=8080 mise run book-serve
```

### Build Failures

Clean and rebuild:

```bash
mise run book-clean
mise run book-build
```

### Permission Errors

Make sure task scripts are executable:

```bash
chmod +x .mise/tasks/book-*
```

## Contributing

We welcome contributions to the documentation! Please:

1. Follow the existing structure and style
2. Test your changes locally: `mise run book-serve`
3. Check for broken links: `mise run book-test`
4. Create clear, focused pull requests

For more information, see the [Contributing Guide](./src/development/contributing.md).

## CI/CD Integration

For continuous integration, add to your workflow:

```yaml
# Install and build documentation
- run: mise run book-install
- run: mise run book-build
- run: mise run book-test

# Deploy (on main branch)
- run: mise run book-deploy
  if: github.ref == 'refs/heads/main'
```

## Resources

- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [Markdown Guide](https://www.markdownguide.org/)
- [KDL Documentation](https://kdl.dev/)
- [Mise Documentation](https://mise.jdx.dev/)

## License

The documentation is licensed under the same terms as the Sentinel project (MIT License).