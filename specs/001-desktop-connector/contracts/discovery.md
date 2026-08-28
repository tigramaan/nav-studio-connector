# DNS-SD discovery contract v1

Service type: `_umec-nav._tcp.local.`

Required resolved fields: instance name, hostname, at least one IP address and non-zero port.

Recognized TXT keys:

- `device_id`: stable opaque robot identifier; otherwise a deterministic diagnostic identifier is derived from normalized instance and hostname.
- `schema`: discovery contract version.
- `model`: public model projection.
- `api_version`: Studio API contract version.
- `path`: Studio path beginning with `/`.
- `url`: absolute `https://` Studio URL whose host matches the record hostname or resolved address.
- `identity`: exact `receipt-v1` marker when a signed receipt is available.
- `receipt`: bounded signed identity receipt matching `identity-receipt-v1.md`; verified only against an active pinned trust root.

Unknown keys are ignored. Invalid records are returned only as sanitized diagnostic errors and never as selectable robots.
