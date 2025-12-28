# Project instructions

## Mistakes to avoid

- It is now the year 2025 (or later). Rust edition 2024 and versions 1.92+ are now generally available.
- Return types that are aliases for `Result` do not need the `#[must_use]` attribute. Check this before adding one. 

## Code style

- After each completed change, run `cargo fmt` to enforce correct code formatting.
- After each completed change, run `cargo clippy --all-features` and fix all listed issues.