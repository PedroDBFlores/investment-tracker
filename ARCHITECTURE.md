# Architecture

## Overview

`investment-tracker` is an offline-first CLI tool for managing a personal investment portfolio. It is intentionally dependency-light and requires no network access or database daemon.

---

## Module Structure

```
src/
├── main.rs              # Binary entry point — parses CLI args and dispatches
├── lib.rs               # Library root — re-exports core, error, utils for benchmarks/tests
├── error.rs             # InvestmentError enum + anyhow::Result alias
├── core/
│   ├── models.rs        # Domain types: Investment, PriceEntry, DividendEntry, SaleEntry, InvestmentType
│   ├── storage.rs       # StorageBackend trait + JSON-backed Storage implementation
│   ├── portfolio.rs     # PortfolioAnalytics<S> — aggregated portfolio statistics
│   └── config.rs        # App configuration (currency, date format, data directory)
├── cli/
│   └── commands/        # One file per subcommand (add, list, view, update, delete, …)
└── utils/
    └── display.rs       # Formatting helpers: fmt_amount, sparkline, colours, timestamps
```

---

## Key Design Decisions

### JSON-first storage

All investment data is stored in a single pretty-printed JSON file (default: `~/.investment_tracker/investments.json`). The path is overridable via the `INVESTMENT_TRACKER_DATA` environment variable or the config file.

**Why JSON over SQLite?**
- Zero setup — no migrations, no schema versioning at launch.
- Fully human-readable and directly editable in any text editor.
- Easy to back up, diff, and version-control.
- Sufficient performance for the expected portfolio size (< 10 000 entries, typically < 1 000).

SQLite remains a deferred option; the `StorageBackend` trait (see below) makes the migration non-breaking.

### `StorageBackend` trait

`src/core/storage.rs` defines a `StorageBackend` trait that covers all CRUD operations. `Storage` (JSON) implements it. `PortfolioAnalytics<S: StorageBackend>` is generic over the backend.

Adding a new backend (e.g. SQLite) requires only:
1. `impl StorageBackend for SqliteStorage { … }`
2. Pass the new type wherever `Storage::open()` is currently called.

No command code needs to change.

### Error handling

The crate uses a two-layer error strategy:

- **`InvestmentError`** (defined in `error.rs`) covers domain-level failures:  `InvalidAmount`, `InvalidDate`, `NotFound`, `InsufficientUnits`. These are always user-actionable (bad input, missing record).
- **`anyhow::Error`** wraps everything else — I/O errors, JSON parse failures — and carries context via `.with_context(…)` so the error message always names the file and operation that failed.

All public functions return `crate::error::Result<T>` which is an alias for `anyhow::Result<T>`. This lets callers use `?` freely while still being able to `downcast_ref::<InvestmentError>()` when they need to match domain errors specifically.

### Atomic config writes

`Config::save()` writes to `config.json.tmp` in the same directory, then renames it over `config.json`. Because `rename` is atomic on all POSIX systems (and atomic on Windows for files on the same volume), a crash mid-write never leaves a corrupt config file.

### Platform-specific code

`libc` is declared as a `[target.'cfg(unix)'.dependencies]` dependency. The only Unix-specific code is `drain_tty_input()` in `interactive.rs`, which is gated with `#[cfg(unix)]` / `#[cfg(not(unix))]`. A no-op stub is provided for non-Unix platforms so the binary compiles on Windows.

---

## Data Flow

```
CLI args (clap)
      │
      ▼
commands/mod.rs  ──execute()──►  commands/<name>.rs
                                       │
                              core::Storage (JSON)
                                       │
                              core::models (domain types)
                                       │
                              utils::display (formatting)
                                       │
                                  stdout / stderr
```

---

## Performance Characteristics

| Operation         | Complexity | Typical latency (1 000 records) |
|-------------------|------------|----------------------------------|
| Load all          | O(n)       | ~5 ms                            |
| Add one           | O(n)       | ~10 ms (load + save)             |
| Bulk add k        | O(n + k)   | ~10–50 ms                        |
| Delete / update   | O(n)       | ~10 ms                           |
| Portfolio summary | O(n)       | ~5 ms                            |

The load-all/save-all pattern is intentional simplicity. For portfolios beyond ~10 000 entries, streaming JSON or a SQLite backend should be considered (see `StorageBackend`).

Performance benchmarks live in `benches/portfolio.rs` and can be run with:

```
cargo bench
```

---

## Testing Strategy

- **Unit tests** in each module's `#[cfg(test)]` block cover domain logic, validation edge cases, and storage operations including error scenarios (corrupt JSON, missing directories, ambiguous ID prefixes).
- **Integration tests** in `tests/cli_integration.rs` drive the compiled binary end-to-end via `assert_cmd`, covering full round-trips: add → list → view → update → delete, CSV/JSON import-export, and price/dividend history.

Run all tests with:

```
cargo test
```
