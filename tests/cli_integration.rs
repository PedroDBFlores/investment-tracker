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
        .args(["delete", &id])
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
    .stderr(predicate::str::contains(
        "Date must be in YYYY-MM-DD format",
    ));

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
