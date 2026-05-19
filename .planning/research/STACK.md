# Stack Additions & Compatibility Review

## Stack additions

- Keep existing Rust core for parser and file operations:
  - `winnow` for query parsing remains the parsing foundation.
  - `clap` and `serde` stack remain unchanged in CLI/TUI layers.
- Add `chrono`-style date math usage only if not already present.
  - If recurring and threshold calculations are already implemented with `chrono` in crate dependencies, prefer reuse.
- No new TUI rendering crates are needed for spacer rows or cursor highlighting.

## Version constraints (recommended)

- Favor versions already in the repo lockfile; add minimal new dependencies and pin to existing lockfile generation patterns.
- Use existing workspace dependency policy:
  - stable Rust
  - minimal feature flags
  - explicit unit tests in crate-local `tests` modules.

## Operational additions

- Add small configuration knobs in `.planning` first, then propagate to CLI/TUI config model:
  - done rotation size threshold
  - done rotation retention count.
- Keep recurring behavior configurable by command-line and config defaults.

## Notes

- For v1.6.3, we should prioritize predictable behavior over adding many external dependencies.
- Most value comes from integrating new features into existing `todotxt-core`, `todotxt-cli`, and `todotxt-tui` boundaries.

