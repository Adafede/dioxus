# Security Policy

## Supported versions

Only the current `main` branch and the latest tagged release are supported for security updates.

## Reporting a vulnerability

Please do not open public issues for vulnerabilities.

Report privately by email to the maintainers listed in the organization profile, including:

- affected crate/app (`lotus-api`, `lotus-explorer`, `shared`, etc.)
- reproduction steps or proof-of-concept
- expected impact and scope
- suggested fix (if available)

You will receive an acknowledgement within 3 business days. We aim to provide a remediation plan within 7 business days.

## Security release process

1. Reproduce and triage the report on a private branch.
2. Patch with tests.
3. Run `make qa` and supply-chain checks.
4. Publish a fixed release and rotate any impacted credentials.
5. Publish an advisory/changelog note after patch availability.

