# secret-scan

**Invariant**:
No commit adds text that matches a known secret format or a high-entropy token assigned to a name such as key, token, secret, or password.

Kit spec: [checks/secret-scan.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/secret-scan.md).

## Implemented by

- Tool: gitleaks 8.30.1, with `--ignore-gitleaks-allow`, so an inline allow comment cannot hide a secret.
- Hook command: `gitleaks git --staged --redact --no-banner --ignore-gitleaks-allow`.
- CI command: the same scan over every commit in the pull request.

Runs in: hook and CI.
