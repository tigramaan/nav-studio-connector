#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
dry_run=false
skip_install=false
for argument in "$@"; do
  case "$argument" in
    --dry-run) dry_run=true ;;
    --skip-install) skip_install=true ;;
    *) echo "Unknown argument: $argument" >&2; exit 2 ;;
  esac
done

run() {
  if "$dry_run"; then printf '%q ' "$@"; printf '\n'; else "$@"; fi
}

cd "$repo_root"
if ! "$skip_install"; then run npm ci --no-audit --no-fund; fi
run npm run build
run cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
run cargo test --manifest-path src-tauri/Cargo.toml
run npm run validate:traceability
run node tools/scan-secrets.mjs
run npm run tauri -- build --bundles deb,appimage
