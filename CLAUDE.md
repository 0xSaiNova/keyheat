# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

KeyHeat is a keyboard analytics daemon written in Rust. Read `keyheat-spec.md` for the full architectural spec before writing any code.

## Build Commands

```bash
cargo build                           # debug build
cargo build --release                 # release build
cargo clippy -- -D warnings           # lint (must pass with no warnings)
cargo fmt                             # format code
cargo test                            # run all tests
cargo test test_name                  # run single test
cargo test --test integration_test    # run specific integration test file
```

Run `cargo clippy -- -D warnings` and `cargo fmt` before every commit.

## Architecture

The daemon follows a modular architecture with clean trait boundaries between components:

- **Listener**: captures keyboard events via evdev
- **Aggregation Engine**: processes raw events into statistics
- **Storage**: persists data to SQLite (WAL mode for concurrent reads during writes)
- **Report Builder**: computes analytics from stored data
- **Renderer**: displays reports via ratatui TUI

Each module should be testable in isolation through trait abstractions.

## Code Standards

This is a public portfolio repo. Every line should look like it was written by a senior engineer.

### Rust Style
- Idiomatic Rust following conventions from ripgrep, fd, tokio
- Performance matters: prefer stack allocation, avoid unnecessary clones, use iterators over collecting into vecs
- Comments explain WHY, not WHAT. If code needs explanation, rewrite it to be clearer.
- No doc comments on private functions unless logic is genuinely non-obvious

### Error Handling
- `thiserror` for library errors, `anyhow` for CLI binary
- No `.unwrap()` in production code paths
- `.expect()` only for genuine programmer errors

### Configuration
- `serde` with TOML deserialization
- Validate at load time, not use time

### Testing
- Unit tests next to code, integration tests in `tests/`
- Descriptive names: `session_ends_after_idle_threshold` not `test_session`
- Cover edge cases, not just happy paths

## Strict Rules

- No emojis anywhere (code, comments, commits, branch names)
- No commented out code
- No AI boilerplate comments like "This module handles..."
- No hyphens or dashes in comments, docs, or commit messages
- Minimal dependencies. Core stack: evdev, rusqlite, ratatui, serde, toml, thiserror, anyhow, clap

## Commit Format

```
type: short description
```

Types: `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, `chore`

Lowercase, imperative, under 72 chars. No emojis. Be specific about what changed.

Good: `feat: add session idle detection with configurable threshold`
Bad: `Update main.rs` or `feat: implement-keyboard-listener`
