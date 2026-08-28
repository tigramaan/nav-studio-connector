# Architecture

```text
GUI / JSON CLI
      ↓
application workflows
      ↓
pure domain guards and trust policy
      ↓
network adapters     platform trust adapters
```

The domain validates HTTPS URLs, fingerprints, timeouts, signed identity receipts and authorization transitions without I/O. A versioned policy file pins Ed25519 public roots by key ID, validity window, usage and revocation state. Network adapters discover `_umec-nav._tcp.local.`, inspect the peer certificate and perform a trusted HTTPS probe. Platform adapters receive an already-authorized operation and mutate only the documented certificate target. Connector-owned operation receipts enable bounded removal.

Trust boundaries: LAN input, the pinned public receipt-root policy, OS trust stores, privilege broker and default browser. mDNS/TXT, DNS, TLS certificates and HTTP responses are untrusted until checked. Receipt verification binds `device_id`, normalized hostname, observed certificate SHA-256 and expiry before trust authorization. Engineering SSH, robot tokens and all signing private keys are outside the process and repository.

Distribution has a separate boundary. Branch artifacts may be unsigned development outputs. A `v*` Windows tag imports an encrypted PFX only into the ephemeral runner, configures Tauri SHA-256 Authenticode plus RFC 3161-compatible timestamping, verifies both produced executables and removes the imported material. Missing or invalid signing inputs stop the tagged build.

Observability consists of bounded error codes, durations, public certificate fingerprints and sanitized diagnostic summaries. Arbitrary response bodies and secret values are not recorded.
