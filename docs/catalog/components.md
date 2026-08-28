# Component catalog

| Component | Purpose | Interface | Dependencies | Errors | REQ / tests |
|---|---|---|---|---|---|
| GuidedConnector | Human discovery, inspection, confirmation and open workflow | Tauri commands | application services | classified UI errors | REQ-003, 005, 009, 011, 017 / viewport E2E |
| AgentGuide | Copyable machine contract | `agent describe --json` | CLI contract | none | REQ-010, 011 / CLI contract test |
| DiagnosticsPanel | Sanitized recovery report | `diagnostics` | diagnostic service | bounded check errors | REQ-014, 016 / diagnostic snapshot |
