# Developer quickstart

Prerequisites: Node.js 22+, Rust stable, platform-specific Tauri build dependencies.

```powershell
npm ci
npm run check
npm run tauri dev
npm run tauri build
```

Agent contract smoke test:

```powershell
cargo run --manifest-path src-tauri/Cargo.toml --bin nav-studio-connector -- agent describe --json
cargo run --manifest-path src-tauri/Cargo.toml --bin nav-studio-connector -- discover --timeout 3 --json
```

Unknown identities must be inspected and independently verified before `trust install` is called.
