# Integration handover

Robot-side integration MUST advertise `_umec-nav._tcp.local.` according to `001-desktop-connector/contracts/discovery.md`. New robot firmware SHOULD publish a UMEC-signed `receipt-v1`; legacy firmware remains usable through explicit human fingerprint comparison. No client distribution may contain engineering SSH credentials.

## Client workflow

1. Discover `_umec-nav._tcp.local.` candidates or accept a guarded absolute `https://` manual address.
2. Inspect the public TLS certificate without treating the transport as authenticated.
3. Authorize the observed SHA-256 only from a valid signed receipt or an independently supplied exact fingerprint.
4. Install the authorized certificate into Windows `CurrentUser\\Root` or Ubuntu system CA store.
5. Run strict HTTPS health with bounded retries, then open Nav Studio in the default browser.

The GUI and JSON CLI call the same application services. Agent mode fails closed for an unknown identity and returns documented exit codes from `001-desktop-connector/contracts/agent-cli.md`.

## Distribution handoff

- Windows release: NSIS installer plus standalone GUI/CLI executable. Branch packages may be unsigned; a `v*` tag requires the encrypted `WINDOWS_CERTIFICATE` and `WINDOWS_CERTIFICATE_PASSWORD` secrets and verifies Authenticode plus timestamp before artifact upload.
- Ubuntu release: `.deb` is primary; AppImage is an optional diagnostic artifact. Trust installation invokes only the fixed `pkexec` helper plan. The package workflow installs the `.deb` on an ephemeral Ubuntu host and verifies GUI startup plus disposable-CA installation/removal.
- `artifacts/` is a local, gitignored handoff directory. GitHub Actions packages are the reproducible public artifacts.
- Public project information is served from `https://tigramaan.github.io/nav-studio-connector/`; the versioned privacy disclosure is available at `/privacy/`, and downloads remain on GitHub Releases.
- Release authorization and SignPath Foundation responsibilities are published at `https://tigramaan.github.io/nav-studio-connector/code-signing/`.

## Outstanding external release input

- A CA-issued organization code-signing PFX and password must be provisioned as GitHub encrypted secrets before the first production `v*` tag. This repository contains and requires no private Authenticode material.
