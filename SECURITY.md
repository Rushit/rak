# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Please do NOT open public issues for security vulnerabilities.**

To report a security vulnerability, please email: **hello@zvectorlabs.com**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

We will respond within 48 hours and provide updates on the resolution timeline.

## Security Best Practices

- Never commit API keys or secrets to the repository
- Always use environment variables or config files (gitignored) for credentials
- Keep dependencies up to date: `cargo audit`

## Known Security Advisories

### RUSTSEC-2023-0071: rsa 0.9.9 (Medium Severity)
- **Status**: No fix available (transitive dependency via sqlx-mysql)
- **Impact**: ZDK does not directly use MySQL; vulnerability is in TLS key exchange
- **Mitigation**: PostgreSQL and SQLite are primary database options
- **Tracking**: Waiting for sqlx to update rsa dependency

### Unmaintained Dependencies
- **fxhash** (via scraper): Warning only, no security impact
- **paste** (via rmcp): Compile-time macro only, no runtime risk

Run `cargo audit` regularly to check for updates.

