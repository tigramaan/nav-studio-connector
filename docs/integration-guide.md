# Nav Studio integration guide

Robot-side Nav Studio must advertise `_umec-nav._tcp.local.` with resolved hostname, address and HTTPS port. Allowlisted TXT keys are `schema`, `device_id`, `model`, `api_version` and optional guarded `path`. The certificate SAN must contain the advertised hostname. Current legacy deployments may use a self-signed leaf certificate after explicit fingerprint confirmation.

Recommended production evolution:

1. assign a fleet-unique `umec-nav-<short-device-id>.local` hostname;
2. serve HTTPS on 443 with a local Device CA/leaf chain;
3. return authenticated device identity and compare `device_id` after TLS;
4. publish a bounded signed identity receipt binding device ID, hostname and certificate fingerprint;
5. rotate certificates through a separately authorized receipt rather than silent replacement.

Do not put session tokens, passwords, SSH keys or full serial payloads in DNS-SD TXT records or QR codes.
