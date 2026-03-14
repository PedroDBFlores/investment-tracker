# 💼 Investment Tracker

A fast, offline-first CLI tool for tracking your investment portfolio. Record investments, monitor performance, track dividends, and analyse your portfolio — all from the terminal, with no internet connection required.

[![CI](https://github.com/pedrodbflores/investment-tracker/actions/workflows/ci.yml/badge.svg)](https://github.com/pedrodbflores/investment-tracker/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
  - [Interactive Mode](#interactive-mode)
  - [Add an Investment](#add-an-investment)
  - [List Investments](#list-investments)
  - [View an Investment](#view-an-investment)
  - [Update an Investment](#update-an-investment)
  - [Delete an Investment](#delete-an-investment)
  - [Record a Price Entry](#record-a-price-entry)
  - [Performance Reports](#performance-reports)
  - [Dividend Tracking](#dividend-tracking)
  - [Portfolio Summary](#portfolio-summary)
  - [Analytics](#analytics)
  - [Export & Import](#export--import)
  - [Configuration](#configuration)
- [Data Storage](#data-storage)
- [Investment Types](#investment-types)
- [Development](#development)
- [License](#license)

---

## Features

- **Interactive mode** — a guided, menu-driven interface for all portfolio actions
- **Investment management** — add, view, update, and delete investments
- **Price history** — manually record price entries and track value over time
- **Performance reports** — time-weighted return (TWR), ROI, and return % per investment or across the whole portfolio, with time range filters (`1m`, `3m`, `6m`, `1y`, `all`)
- **Dividend tracking** — record dividend payments and view total income per investment
- **Portfolio summary** — total invested, current value, overall ROI, and allocation breakdown by investment type
- **Analytics** — best/worst performers, return statistics (mean, std dev, min/max), and top dividend earners
- **Export & Import** — save your portfolio to CSV or JSON, and restore it from either format
- **Configurable** — set your preferred currency, data directory, and more
- **Offline-first** — no API keys, no internet connection required; all data lives locally
- **Colourful terminal output** — colour-coded tables, spinners, and clear feedback throughout

---

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024, stable toolchain)

### Build from source

```sh
git clone https://github.com/yourusername/investment-tracker.git
cd investment-tracker
cargo build --release
```

The compiled binary will be at `./target/release/investment_tracker`.

### Run directly

```sh
cargo run -- <command> [options]
```

### Install to PATH

```sh
cargo install --path .
```

---

## Quick Start

```sh
# Add your first investment (amount becomes the initial current value automatically)
investment_tracker add stock "Apple Inc" 5000.00 2024-01-15 --symbol AAPL

# List all investments
investment_tracker list

# Open the interactive guided menu
investment_tracker interactive
```

---

## Usage

### Interactive Mode

The recommended way to use Investment Tracker. A full-screen menu guides you through every action without needing to remember any command syntax.

```sh
investment_tracker interactive
```

The menu offers:
- ➕ Add a new investment
- 📋 List investments (with total invested and total current value summary)
- ✏️  Update an investment
- 💹 Record a price entry
- 💰 Record a dividend payment
- 🗑️  Delete an investment
- 🚪 Quit

---

### Add an Investment

```sh
investment_tracker add <type> <name> <amount> [date] [options]
```

| Argument | Description |
|---|---|
| `type` | Investment type: `stock`, `etf`, `mutualfund`, `deposit`, `bond`, `crypto`, or any custom string |
| `name` | Name of the investment |
| `amount` | Amount invested (must be greater than 0) |
| `date` | Purchase date in `YYYY-MM-DD` format (defaults to today) |

| Option | Description |
|---|---|
| `-s`, `--symbol` | Ticker symbol (e.g. `AAPL`, `BTC`) |
| `-y`, `--dividend-yield` | Dividend yield percentage (e.g. `3.5` for 3.5%) |
| `-f`, `--dividend-frequency` | Dividend frequency (e.g. `monthly`, `quarterly`, `annual`) |

**Examples:**

```sh
# Stock with ticker symbol
investment_tracker add stock "Apple Inc" 5000.00 2024-01-15 --symbol AAPL

# ETF with dividend information
investment_tracker add etf "Vanguard S&P 500" 10000.00 2024-03-01 --symbol VOO --dividend-yield 1.5 --dividend-frequency quarterly

# Crypto (date defaults to today)
investment_tracker add crypto "Bitcoin" 2500.00 --symbol BTC

# Fixed deposit
investment_tracker add deposit "6-Month Term Deposit" 20000.00 2024-06-01
```

> **Note:** When an investment is created, its current value is automatically initialised to the invested amount. This means it shows a return of 0% until you record a price update.

---

### List Investments

```sh
investment_tracker list
```

Displays a formatted table of all investments with columns for ID, name, type, amount invested, current value, return, and date. A summary line at the bottom shows the total number of investments.

---

### View an Investment

```sh
investment_tracker view <id>
```

Shows full details for a single investment, including its ID, type, symbol, amount, current value, notes, dividend information, and price history entries.

```sh
investment_tracker view a1b2c3d4
```

> **Tip:** IDs are UUIDs. You only need to provide enough characters to match — the `list` command shows the first 8 characters of each ID for convenience.

---

### Update an Investment

```sh
investment_tracker update <id> [options]
```

| Option | Description |
|---|---|
| `amount` | New invested amount (positional) |
| `--current-value` | Manually set the current value |
| `--date` | Update the investment date |

**Examples:**

```sh
# Update the invested amount
investment_tracker update a1b2c3d4 1500.00

# Set a new current value manually
investment_tracker update a1b2c3d4 --current-value 1750.00
```

---

### Delete an Investment

```sh
investment_tracker delete <id>
```

Permanently removes the investment and all its associated price history and dividend records.

---

### Record a Price Entry

Track the value of an investment over time by recording price entries manually.

```sh
investment_tracker add-price <id> <price> [date] [options]
```

| Argument | Description |
|---|---|
| `id` | Investment ID |
| `price` | Current price / value |
| `date` | Date of the price entry in `YYYY-MM-DD` format (defaults to today) |

| Option | Description |
|---|---|
| `-n`, `--notes` | Optional notes for this entry |

**Examples:**

```sh
# Record today's price
investment_tracker add-price a1b2c3d4 5400.00

# Record a historical price with a note
investment_tracker add-price a1b2c3d4 4800.00 2024-06-01 --notes "Post-correction low"
```

Recording two or more price entries enables **time-weighted return (TWR)** calculations in performance reports.

---

### Performance Reports

```sh
# All investments
investment_tracker performance

# Specific investment
investment_tracker performance -- <id>

# With a time range filter
investment_tracker performance --range 3m
investment_tracker performance -- <id> --range 1y
```

**Available time ranges:**

| Range | Period |
|---|---|
| `1m` | Last 30 days |
| `3m` | Last 90 days |
| `6m` | Last 180 days |
| `1y` | Last 365 days |
| `all` | All time (default) |

The report shows invested amount, current value, absolute return, return %, and time-weighted return (TWR) for each investment. When viewing a single investment, a full price history table is also displayed.

---

### Dividend Tracking

#### Record a dividend payment

```sh
investment_tracker add-dividend <id> <amount> [date] [options]
```

```sh
# Record a quarterly dividend
investment_tracker add-dividend a1b2c3d4 87.50 2024-03-31 --notes "Q1 2024 dividend"
```

#### List dividend history

```sh
investment_tracker list-dividends <id>
```

Displays a table of all dividend entries for the investment, along with total dividends received.

---

### Portfolio Summary

```sh
investment_tracker portfolio
```

Displays a high-level summary of your entire portfolio:

- Total number of investments
- Total amount invested
- Total current value
- Total dividends received
- Overall return (absolute and percentage)
- Allocation breakdown by investment type (count, value, and % of portfolio)

---

### Analytics

```sh
investment_tracker analytics
```

Advanced analysis across your whole portfolio:

- **🏆 Best Performers** — top 3 investments by return %
- **📉 Worst Performers** — bottom 3 investments by return %
- **💰 Highest Dividend Earners** — top 3 investments by total dividends received
- **📈 Return Statistics** — mean return, standard deviation, min/max across all investments with current values

---

### Export & Import

#### Export

```sh
# Export to CSV (default)
investment_tracker export portfolio.csv

# Export to JSON
investment_tracker export portfolio.json --format json
```

#### Import

```sh
investment_tracker import portfolio.csv
investment_tracker import portfolio.json
```

The importer accepts both CSV and JSON files. Duplicate detection is applied during import.

---

### Configuration

```sh
# Show current settings
investment_tracker config show

# Change a setting
investment_tracker config set <key> <value>

# Reset to defaults
investment_tracker config reset
```

**Available configuration keys:**

| Key | Description | Default |
|---|---|---|
| `currency` | Currency code (e.g. `USD`, `EUR`, `GBP`) | `USD` |
| `data-directory` | Custom path for storing `investments.json` | `~/.investment_tracker/` |
| `date-format` | Preferred date display format | `YYYY-MM-DD` |
| `show-dividends` | Show dividend data in outputs (`true`/`false`) | `true` |
| `color-output` | Enable coloured terminal output (`true`/`false`) | `true` |

**Supported currency symbols:**

`USD` ($), `EUR` (€), `GBP` (£), `JPY` (¥), `CHF` (Fr), `CAD` (CA$), `AUD` (A$), `BRL` (R$), `INR` (₹), `KRW` (₩), `BTC` (₿), `ETH` (Ξ), and many more. Unknown codes fall back to displaying the code itself.

**Examples:**

```sh
investment_tracker config set currency EUR
investment_tracker config set data-directory /home/user/documents/investments
investment_tracker config set color-output false
```

Configuration is stored at `~/.config/investment_tracker/config.json`.

---

## Data Storage

All investment data is stored as a human-readable JSON file at:

```
~/.investment_tracker/investments.json
```

You can override this path using the `data-directory` config key, or by setting the `INVESTMENT_TRACKER_DATA` environment variable (useful for testing or managing multiple portfolios):

```sh
INVESTMENT_TRACKER_DATA=/path/to/my-portfolio.json investment_tracker list
```

---

## Investment Types

| Type | CLI value |
|---|---|
| Stock | `stock` |
| ETF | `etf` |
| Mutual Fund | `mutualfund` |
| Fixed Deposit | `deposit` |
| Bond | `bond` |
| Cryptocurrency | `crypto` |
| Other / Custom | `other` (or any string) |

---

## Development

### Running tests

```sh
# All tests (unit + integration)
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test cli_integration
```

### Project structure

```
src/
├── main.rs                  # Entry point
├── error.rs                 # Centralised error types
├── cli/
│   └── commands/
│       ├── mod.rs           # Clap CLI definition and dispatch
│       ├── interactive.rs   # Guided interactive menu
│       ├── add.rs
│       ├── list.rs
│       ├── view.rs
│       ├── update.rs
│       ├── delete.rs
│       ├── add_price.rs
│       ├── performance.rs
│       ├── add_dividend.rs
│       ├── list_dividends.rs
│       ├── portfolio.rs
│       ├── analytics.rs
│       ├── export.rs
│       ├── import.rs
│       └── config.rs
├── core/
│   ├── models.rs            # Investment, PriceEntry, DividendEntry
│   ├── storage.rs           # JSON persistence layer
│   ├── portfolio.rs         # Portfolio analytics engine
│   └── config.rs            # Configuration management
└── utils/
    └── display.rs           # Formatting helpers (currency, spinners)
```

### Key dependencies

| Crate | Purpose |
|---|---|
| `clap` | CLI argument parsing |
| `dialoguer` | Interactive prompts and menus |
| `comfy-table` | Rich terminal table rendering |
| `serde` / `serde_json` | Data serialisation |
| `chrono` | Date handling |
| `uuid` | Investment ID generation |
| `indicatif` | Progress spinners |
| `csv` | CSV export/import |
| `dirs` | Platform-appropriate file paths |
| `libc` | Unix tty drain for interactive mode |

---

## License

This project is licensed under the [MIT License](./LICENSE).