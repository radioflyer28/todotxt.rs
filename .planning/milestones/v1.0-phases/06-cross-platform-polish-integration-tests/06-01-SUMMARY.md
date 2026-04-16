---
plan: 06-01
phase: 06
status: complete
commit: f627692
---

# 06-01 Summary — Compiler Hardening

## One-liner
Added `#![deny(warnings)]` to both crate roots; all `.unwrap()` calls in core confirmed test-only — zero production panics.

## What was done
- Added `#![deny(warnings)]` as the first line of `crates/todotxt-core/src/lib.rs`
- Added `#![deny(warnings)]` as the first line of `crates/todotxt-cli/src/main.rs`
- Audited all 5 `.unwrap()` calls in `todotxt-core/src/` — all in `#[cfg(test)]` modules (`filter.rs` lines 152-153, `portable.rs` lines 24-32)
- Build with `#![deny(warnings)]` produced zero warnings — no suppressions needed

## Verification
- `cargo build --workspace` → exit 0, zero warnings
- `cargo clippy --workspace -- -D warnings` → exit 0, zero warnings
- `cargo test -p todotxt-core` → exit 0, all tests pass
- Zero `.unwrap()` in non-test library code confirmed

## Files changed
- `crates/todotxt-core/src/lib.rs` — `#![deny(warnings)]` added
- `crates/todotxt-cli/src/main.rs` — `#![deny(warnings)]` added
