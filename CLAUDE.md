# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

libpgfmt is a Rust library for formatting PostgreSQL-specific SQL and PL/pgSQL.

## Build Commands

```sh
cargo build
cargo test
cargo test <test_name>    # run a single test
cargo clippy              # lint
cargo fmt --check         # check formatting
cargo fmt                 # auto-format
```
