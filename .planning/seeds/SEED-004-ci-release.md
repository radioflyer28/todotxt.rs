---
planted_during: v1.0
trigger_when: Core + CLI is complete and usable; ready to ship to other users
---

# SEED-004: CI/CD Pipeline + Release Binaries

## Idea

Set up GitHub Actions CI matrix to build and test on Windows, Linux, and macOS, and publish release binaries via GitHub Releases (or `cargo dist`).

## Why This Matters

Cross-platform is the primary reason for the Rust port. Without CI, cross-platform compatibility can't be verified continuously. Release binaries let non-Rust users install the tool without `cargo install`.

## When to Surface

- v1.0 Core + CLI milestone is feature-complete
- Ready to share with other users or publish to crates.io

## Scope Ideas

- GitHub Actions: build + test matrix (Windows/Linux/macOS, stable Rust)
- Clippy + rustfmt in CI
- Release binaries via `cargo dist` or manual artifact upload
- Optional: publish `todotxt-core` to crates.io
- Optional: Homebrew formula, apt/deb package, winget manifest
