# Dependency Management

Use Bun for TypeScript/Svelte dependencies and Cargo for Rust dependencies.

## TypeScript

- Install runtime dependency: `bun add <package>`
- Install dev/build/test tool: `bun add -d <package>`
- Update dependencies: `bun update` or `bun update <package>`
- Check outdated packages: `bun outdated`
- Verify after changes: `bun run check`

Put packages imported by app runtime code in `dependencies`. Put CLI, build, check, test, and tooling packages in `devDependencies`.

## Rust

Run Cargo commands from `src-tauri/`.

- Install runtime crate: `cargo add <crate>`
- Install test-only crate: `cargo add --dev <crate>`
- Verify after changes:
  - `cargo check --locked`
  - `cargo test --locked`
  - `cargo clippy --all-targets --all-features --locked -- -D warnings`

Use `--locked` for normal verification so `Cargo.lock` is not changed unexpectedly. Omit `--locked` only when intentionally updating Rust dependencies.

## Tauri Plugins

Tauri plugins usually need both sides:

- TypeScript package: `bun add @tauri-apps/plugin-<name>`
- Rust crate from `src-tauri/`: `cargo add tauri-plugin-<name>`

Then register the plugin in Rust and add required permissions in `src-tauri/capabilities/` when the plugin requires them.
