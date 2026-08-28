# View catalog

| View/state | Purpose | Inputs/actions | Errors | REQ / tests |
|---|---|---|---|---|
| Search | Start bounded mDNS or manual HTTPS fallback | timeout, guarded URL | discovery/URL errors | REQ-001, 002, 016, 017 |
| Candidate list | Select an untrusted candidate | DNS-SD projection | invalid records excluded | REQ-003, 011 |
| Certificate verification | Show identity and require independent comparison | full SHA-256 + last-8 confirmation | mismatch/confirmation required | REQ-003–006 |
| Ready | Open only after trust and HTTPS health | verified URL | health/open errors | REQ-009 |
| Diagnostics | Copy sanitized checks | optional URL | per-check codes | REQ-014, 016 |
