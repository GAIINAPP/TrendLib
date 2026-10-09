# Security policy

## Supported versions

TrendLib is pre-1.0. Only the latest released version receives fixes.

| Version | Supported |
| --- | --- |
| latest release | yes |
| anything older | no |

## Reporting a vulnerability

Report privately through GitHub: open the repository's **Security** tab and
choose **Report a vulnerability** (GitHub private vulnerability reporting).
Please do not open a public issue for a security problem.

Include what you have: affected version, platform, a minimal reproduction, and
the impact as you see it. You will get an acknowledgement within 5 working days
and an assessment within 15. If the report is accepted, we agree a disclosure
date with you and credit you in the release notes unless you prefer otherwise.

## Scope

In scope: memory safety or soundness problems in the Rust core or the Python
bindings, crashes reachable from untrusted array input, and supply-chain issues
in the published wheels or sdist.

Out of scope: numerical disagreements with another library (open a normal
issue), denial of service from deliberately huge inputs, and anything in the
caller's own data loaders, which live outside this library by design.

The core crate carries `#![forbid(unsafe_code)]` and has no runtime
dependencies, and CI runs `cargo deny check` on every pull request.
