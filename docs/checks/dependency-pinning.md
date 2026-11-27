# dependency-pinning

**Invariant**:
Every direct dependency has an exact version, and the lockfile is committed and matches the manifest.

Kit spec: [checks/dependency-pinning.md](https://github.com/avidianity/factory/blob/v0.9.0/checks/dependency-pinning.md).

## Implemented by

- Command: `cargo xtask dependency-pinning`.
- Every registry dependency uses an exact `=X.Y.Z` requirement; workspace path dependencies are exempt.
- `cargo metadata --locked` fails when `Cargo.lock` does not match the manifests.
- The Rust toolchain is pinned in `rust-toolchain.toml`, and CI actions are pinned to commit SHAs.

Runs in: hook and CI.
