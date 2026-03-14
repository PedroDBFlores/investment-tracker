use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_add_and_list_investments() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let mut cmd = Command::cargo_bin("investment_tracker")?;

    cmd.args(["add", "stock", "Test Company", "1000.00", "2024-01-15"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Added investment: Test Company"))
        .stdout(predicate::str::contains("Type: Stock"));

    let mut list_cmd = Command::cargo_bin("investment_tracker")?;
    list_cmd
        .args(["list"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Test Company"))
        .stdout(predicate::str::contains("1000.00"))
        .stdout(predicate::str::contains("2024-01-15"));

    Ok(())
}

#[test]
fn test_view_investment() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let mut add_cmd = Command::cargo_bin("investment_tracker")?;
    add_cmd
        .args(["add", "etf", "Test ETF", "5000.00", "2024-02-20"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    let add_output = add_cmd.output()?;
    let output_str = String::from_utf8_lossy(&add_output.stdout);
    let id = extract_id_from_output(&output_str);

    let mut view_cmd = Command::cargo_bin("investment_tracker")?;
    view_cmd
        .args(["view", &id])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Investment Details:"))
        .stdout(predicate::str::contains("Test ETF"))
        .stdout(predicate::str::contains("ETF"))
        .stdout(predicate::str::contains("5000.00"));

    Ok(())
}

#[test]
fn test_update_investment() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let mut add_cmd = Command::cargo_bin("investment_tracker")?;
    add_cmd
        .args(["add", "stock", "Update Test", "1000.00", "2024-01-15"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    let add_output = add_cmd.output()?;
    let output_str = String::from_utf8_lossy(&add_output.stdout);
    let id = extract_id_from_output(&output_str);

    let mut update_cmd = Command::cargo_bin("investment_tracker")?;
    update_cmd
        .args(["update", &id, "1500.00"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated investment:"));

    let mut view_cmd = Command::cargo_bin("investment_tracker")?;
    view_cmd
        .args(["view", &id])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("1500.00"));

    Ok(())
}

#[test]
fn test_delete_investment() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let _ = fs::remove_file(&data_file);

    let add_output = Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Delete Test", "1000.00", "2024-01-15"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(add_output.status.success());
    let output_str = String::from_utf8_lossy(&add_output.stdout);
    let id = extract_id_from_output(&output_str);
    eprintln!("Added investment with ID: {}", id);

    let mut list_cmd = Command::cargo_bin("investment_tracker")?;
    list_cmd
        .args(["list"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Delete Test"));

    let delete_output = Command::cargo_bin("investment_tracker")?
        .args(["delete", &id, "--yes"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    eprintln!(
        "Delete command stdout: {}",
        String::from_utf8_lossy(&delete_output.stdout)
    );
    eprintln!(
        "Delete command stderr: {}",
        String::from_utf8_lossy(&delete_output.stderr)
    );
    eprintln!("Delete command status: {}", delete_output.status);

    assert!(delete_output.status.success());
    assert!(String::from_utf8_lossy(&delete_output.stdout).contains("Deleted investment:"));

    let mut list_after_delete = Command::cargo_bin("investment_tracker")?;
    let result = list_after_delete
        .args(["list"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    let output = result.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!("Delete test output: {}", stdout);
    assert!(stdout.contains("No investments found") || stdout.contains("Total: 0 investments"));

    Ok(())
}

#[test]
fn test_invalid_amount() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let mut cmd = Command::cargo_bin("investment_tracker")?;
    cmd.args(["add", "stock", "Invalid Test", "0.0", "2024-01-15"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Amount must be greater than 0"));

    Ok(())
}

#[test]
fn test_invalid_date() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let mut cmd = Command::cargo_bin("investment_tracker")?;
    cmd.args([
        "add",
        "stock",
        "Invalid Date Test",
        "1000.00",
        "invalid-date",
    ])
    .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
    .assert()
    .failure()
    .stderr(predicate::str::contains("is not a valid date"));

    Ok(())
}

#[test]
fn test_add_dividend_and_list_dividends() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let add_output = Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Dividend Corp", "2000.00", "2024-01-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(add_output.status.success());
    let output_str = String::from_utf8_lossy(&add_output.stdout);
    let id = extract_id_from_output(&output_str);

    // Record a first dividend — check amount and name without currency prefix
    Command::cargo_bin("investment_tracker")?
        .args([
            "add-dividend",
            &id,
            "50.00",
            "2024-03-31",
            "--notes",
            "Q1 dividend",
        ])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Recorded dividend"))
        .stdout(predicate::str::contains("50.00"))
        .stdout(predicate::str::contains("Dividend Corp"))
        .stdout(predicate::str::contains("Dividend entries: 1"));

    // Record a second dividend
    Command::cargo_bin("investment_tracker")?
        .args(["add-dividend", &id, "55.00", "2024-06-30"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Recorded dividend"))
        .stdout(predicate::str::contains("55.00"))
        .stdout(predicate::str::contains("105.00"))
        .stdout(predicate::str::contains("Dividend entries: 2"));

    // List dividends — amounts appear in table without currency prefix dependency
    Command::cargo_bin("investment_tracker")?
        .args(["list-dividends", &id])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Dividend Corp"))
        .stdout(predicate::str::contains("50.00"))
        .stdout(predicate::str::contains("55.00"))
        .stdout(predicate::str::contains("105.00"))
        .stdout(predicate::str::contains("2 dividend payment(s) recorded"));

    Ok(())
}

#[test]
fn test_add_with_symbol() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let add_output = Command::cargo_bin("investment_tracker")?
        .args([
            "add",
            "stock",
            "Apple Inc",
            "5000.00",
            "2024-01-15",
            "--symbol",
            "AAPL",
        ])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(add_output.status.success());
    let output_str = String::from_utf8_lossy(&add_output.stdout);
    assert!(
        output_str.contains("Symbol: AAPL"),
        "Expected 'Symbol: AAPL' in add output, got: {}",
        output_str
    );

    let id = extract_id_from_output(&output_str);

    Command::cargo_bin("investment_tracker")?
        .args(["view", &id])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Apple Inc"))
        .stdout(predicate::str::contains("Symbol: AAPL"));

    Ok(())
}

#[test]
fn test_list_table_format() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Alpha Corp", "1000.00", "2024-01-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    Command::cargo_bin("investment_tracker")?
        .args(["add", "etf", "Beta Fund", "3000.00", "2024-02-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    // Check investment names and numeric amounts appear; currency prefix is config-dependent
    Command::cargo_bin("investment_tracker")?
        .args(["list"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Alpha Corp"))
        .stdout(predicate::str::contains("Beta Fund"))
        .stdout(predicate::str::contains("1000.00"))
        .stdout(predicate::str::contains("3000.00"));

    Ok(())
}

#[test]
fn test_add_price_and_performance() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let add_output = Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "PerfCo", "1000.00", "2024-01-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(add_output.status.success());
    let output_str = String::from_utf8_lossy(&add_output.stdout);
    let id = extract_id_from_output(&output_str);

    // Record a first price entry — check label and amount without currency prefix
    Command::cargo_bin("investment_tracker")?
        .args([
            "add-price",
            &id,
            "800.00",
            "2024-03-01",
            "--notes",
            "Q1 dip",
        ])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Recorded price"))
        .stdout(predicate::str::contains("800.00"))
        .stdout(predicate::str::contains("PerfCo"))
        .stdout(predicate::str::contains("Price history: 1 entries"));

    // Record a second price entry → TWR should be +50%
    Command::cargo_bin("investment_tracker")?
        .args(["add-price", &id, "1200.00", "2024-06-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Recorded price"))
        .stdout(predicate::str::contains("1200.00"))
        .stdout(predicate::str::contains("Price history: 2 entries"))
        .stdout(predicate::str::contains("Time-weighted return: 50.00%"));

    // Detailed performance for the specific investment
    Command::cargo_bin("investment_tracker")?
        .args(["performance", "--", &id])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("PerfCo"))
        .stdout(predicate::str::contains("1000.00"))
        .stdout(predicate::str::contains("1200.00"));

    // Performance summary for all investments
    Command::cargo_bin("investment_tracker")?
        .args(["performance"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("PerfCo"))
        .stdout(predicate::str::contains("Performance Report"));

    Ok(())
}

fn extract_id_from_output(output: &str) -> String {
    // Extract UUID from output like "Added investment: Test Company (uuid)"
    let start = output.find('(').unwrap_or(0) + 1;
    let end = output.find(')').unwrap_or(output.len());
    output[start..end].trim().to_string()
}

// ── Issue 1: current_value equals amount on creation ─────────────────────────

/// When `add` is called, the saved JSON must have current_value == amount.
/// We verify this by reading the raw data file after adding an investment.
#[test]
fn test_add_sets_current_value_equal_to_amount() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "CurrentVal Corp", "3500.00", "2024-05-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    let raw = std::fs::read_to_string(&data_file)?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)?;
    let inv = &parsed[0];

    let amount = inv["amount"].as_f64().expect("amount should be a number");
    let current_value = inv["current_value"]
        .as_f64()
        .expect("current_value should be set, not null");

    assert!(
        (amount - 3500.0).abs() < f64::EPSILON,
        "amount should be 3500, got {}",
        amount
    );
    assert_eq!(
        amount, current_value,
        "current_value ({}) should equal amount ({}) at creation",
        current_value, amount
    );

    Ok(())
}

/// current_value must not be null / missing in the persisted JSON.
#[test]
fn test_add_current_value_is_not_null() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    Command::cargo_bin("investment_tracker")?
        .args(["add", "etf", "No-Null ETF", "1200.00", "2024-03-10"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    let raw = std::fs::read_to_string(&data_file)?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)?;
    let inv = &parsed[0];

    assert!(
        !inv["current_value"].is_null(),
        "current_value should not be null after creation, got: {}",
        inv["current_value"]
    );

    Ok(())
}

/// After creation, the `view` command should show the current value column
/// populated with the same figure as the invested amount.
#[test]
fn test_view_shows_current_value_equal_to_amount_after_add()
-> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let add_output = Command::cargo_bin("investment_tracker")?
        .args(["add", "bond", "View Bond", "800.00", "2024-07-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(add_output.status.success());
    let id = extract_id_from_output(&String::from_utf8_lossy(&add_output.stdout));

    // `view` should show 800.00 in both the invested and current-value fields.
    Command::cargo_bin("investment_tracker")?
        .args(["view", &id])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("800.00"));

    Ok(())
}

/// The `list` table must display the current value for a brand-new investment
/// (previously it showed "—" because current_value was None).
#[test]
fn test_list_shows_current_value_for_new_investment() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    Command::cargo_bin("investment_tracker")?
        .args(["add", "crypto", "ListCoin", "2000.00", "2024-08-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    // The list table should show 2000.00 in the Current Value column, not "—".
    let list_output = Command::cargo_bin("investment_tracker")?
        .args(["list"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(list_output.status.success());
    let stdout = String::from_utf8_lossy(&list_output.stdout);

    assert!(
        !stdout.contains('—'),
        "list should not show '—' in Current Value for a new investment; got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("2000.00"),
        "list should show 2000.00 as current value; got:\n{}",
        stdout
    );

    Ok(())
}

/// Adding multiple investments and then listing them should show each
/// investment's current value equal to its invested amount.
#[test]
fn test_multiple_new_investments_all_have_current_value() -> Result<(), Box<dyn std::error::Error>>
{
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Alpha", "1000.00", "2024-01-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    Command::cargo_bin("investment_tracker")?
        .args(["add", "etf", "Beta", "2500.00", "2024-02-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    Command::cargo_bin("investment_tracker")?
        .args(["add", "bond", "Gamma", "500.00", "2024-03-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success();

    let raw = std::fs::read_to_string(&data_file)?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)?;
    let investments = parsed.as_array().expect("should be a JSON array");

    assert_eq!(investments.len(), 3);

    for inv in investments {
        let amount = inv["amount"]
            .as_f64()
            .expect("each investment must have an amount");
        let current_value = inv["current_value"]
            .as_f64()
            .expect("each investment must have a non-null current_value");
        assert_eq!(
            amount, current_value,
            "investment '{}': current_value {} should equal amount {}",
            inv["name"], current_value, amount
        );
    }

    Ok(())
}

// ── Item 3: strict date validation ───────────────────────────────────────────

/// Dates that look structurally plausible but are calendar-impossible (e.g.
/// month 13, day 32, Feb 29 in a non-leap year) must now be rejected.
#[test]
fn test_add_rejects_impossible_date_month_13() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Bad Date Corp", "1000.00", "2024-13-01"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .failure();

    Ok(())
}

#[test]
fn test_add_rejects_impossible_date_day_32() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Bad Date Corp", "1000.00", "2024-01-32"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .failure();

    Ok(())
}

#[test]
fn test_add_rejects_feb29_on_non_leap_year() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Bad Date Corp", "1000.00", "2023-02-29"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .failure();

    Ok(())
}

/// Feb 29 on an actual leap year must succeed.
#[test]
fn test_add_accepts_feb29_on_leap_year() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Leap Year Corp", "1000.00", "2024-02-29"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Leap Year Corp"));

    Ok(())
}

/// The previously-passing 9999-99-99 must now be rejected.
#[test]
fn test_add_rejects_9999_99_99() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Bad Date Corp", "1000.00", "9999-99-99"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .failure();

    Ok(())
}

// ── Item 5: notes on add and update ──────────────────────────────────────────

/// Notes passed via --notes on `add` should be persisted and shown by `view`.
#[test]
fn test_add_with_notes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let add_output = Command::cargo_bin("investment_tracker")?
        .args([
            "add",
            "stock",
            "Noted Corp",
            "1000.00",
            "2024-01-15",
            "--notes",
            "My first note",
        ])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(add_output.status.success());
    let stdout = String::from_utf8_lossy(&add_output.stdout);
    assert!(
        stdout.contains("My first note"),
        "add confirmation should echo the note; got:\n{}",
        stdout
    );

    // Verify notes are persisted in JSON
    let raw = std::fs::read_to_string(&data_file)?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)?;
    assert_eq!(parsed[0]["notes"], "My first note");

    Ok(())
}

/// Notes passed via --notes on `update` should overwrite the existing notes.
#[test]
fn test_update_notes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let add_output = Command::cargo_bin("investment_tracker")?
        .args([
            "add",
            "stock",
            "Notes Update Corp",
            "1000.00",
            "2024-01-15",
            "--notes",
            "Original note",
        ])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(add_output.status.success());
    let id = extract_id_from_output(&String::from_utf8_lossy(&add_output.stdout));

    Command::cargo_bin("investment_tracker")?
        .args(["update", &id, "--notes", "Updated note"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated note"));

    // Verify the updated note is persisted in JSON
    let raw = std::fs::read_to_string(&data_file)?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)?;
    assert_eq!(parsed[0]["notes"], "Updated note");

    Ok(())
}

/// Notes added on `add` must appear in the `view` output.
#[test]
fn test_view_shows_notes() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let add_output = Command::cargo_bin("investment_tracker")?
        .args([
            "add",
            "etf",
            "View Notes ETF",
            "2000.00",
            "2024-03-01",
            "--notes",
            "Check this note",
        ])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(add_output.status.success());
    let id = extract_id_from_output(&String::from_utf8_lossy(&add_output.stdout));

    Command::cargo_bin("investment_tracker")?
        .args(["view", &id])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Check this note"));

    Ok(())
}

// ── Item 8: delete requires confirmation ─────────────────────────────────────

/// Without --yes and outside a TTY, the prompt falls back to "not confirmed"
/// and the investment must remain intact.
#[test]
fn test_delete_cancelled_by_user() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let add_output = Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Delete Me Not", "1000.00", "2024-01-15"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(add_output.status.success());
    let id = extract_id_from_output(&String::from_utf8_lossy(&add_output.stdout));

    // No --yes flag and no TTY → interact_opt() returns None → treated as cancelled
    Command::cargo_bin("investment_tracker")?
        .args(["delete", &id])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Cancelled"));

    // Investment must still be present
    Command::cargo_bin("investment_tracker")?
        .args(["list"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Delete Me Not"));

    Ok(())
}

/// Passing --yes must skip the confirmation prompt and delete immediately.
#[test]
fn test_delete_confirmed_by_user() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = tempdir()?;
    let data_file = temp_dir.path().join("test_investments.json");

    let add_output = Command::cargo_bin("investment_tracker")?
        .args(["add", "stock", "Delete Me Yes", "1000.00", "2024-01-15"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .output()?;

    assert!(add_output.status.success());
    let id = extract_id_from_output(&String::from_utf8_lossy(&add_output.stdout));

    // --yes bypasses the confirmation prompt entirely
    Command::cargo_bin("investment_tracker")?
        .args(["delete", &id, "--yes"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("Deleted investment:"));

    // Investment must be gone
    Command::cargo_bin("investment_tracker")?
        .args(["list"])
        .env("INVESTMENT_TRACKER_DATA", data_file.to_str().unwrap())
        .assert()
        .success()
        .stdout(
            predicate::str::contains("No investments found")
                .or(predicate::str::contains("Delete Me Yes").not()),
        );

    Ok(())
}
