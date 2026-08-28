# Privacy Policy

Last updated: August 28, 2026

UMEC Nav Studio Connector does not send usage telemetry to the maintainer and does not operate a cloud backend. Its purpose is to communicate with Nav Studio instances on the user's local network.

## Data processed by the application

The application processes local DNS-SD/mDNS service records, host names, IP addresses, ports, HTTPS URLs, publicly presented TLS certificates and their SHA-256 fingerprints, and bounded HTTPS readiness results. This processing is required to discover Nav Studio and verify the identity selected by the user or calling agent.

## Data stored on the device

The application may store non-secret local operation receipts needed to identify and remove certificates that it installed. Diagnostics are local and bounded. They are designed to exclude authentication tokens, passwords, private keys, and arbitrary response bodies.

## Data collected by the maintainer

The maintainer does not collect application telemetry, discovery results, certificates, fingerprints, robot addresses, or diagnostic reports. The application has no maintainer-operated analytics or advertising service.

## Project website

The project website is static and uses no analytics, advertising, cookies, forms, or external runtime assets. It is hosted by GitHub Pages. GitHub may process standard network and security logs under its own privacy statement.

## Third-party destinations

Links to GitHub, release downloads, and documentation leave the project website. Their operators process requests under their respective policies. Connecting to a Nav Studio instance sends requests only to the destination explicitly discovered or supplied locally.

## Security and retention

No private signing keys are distributed with the application. Local receipts remain on the device until cleanup or uninstallation. Removing the application does not automatically remove data independently retained by the operating system or GitHub.

## Changes and contact

Material changes will be published in this repository and reflected by the date above. Questions or privacy requests can be submitted through the project's public [GitHub Issues](https://github.com/tigramaan/nav-studio-connector/issues).
