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

The domain validates HTTPS URLs, fingerprints, timeouts and authorization transitions without I/O. Network adapters discover `_umec-nav._tcp.local.`, inspect the peer certificate and perform a trusted HTTPS probe. Platform adapters receive an already-authorized operation and mutate only the documented certificate target. Connector-owned receipts enable bounded removal.

Trust boundaries: LAN input, OS trust stores, privilege broker and default browser. mDNS/TXT, DNS, TLS certificates and HTTP responses are untrusted until checked. Engineering SSH, robot tokens and private keys are outside the process.

Observability consists of bounded error codes, durations, public certificate fingerprints and sanitized diagnostic summaries. Arbitrary response bodies and secret values are not recorded.
