use super::ConfigCommands;
use crate::core::config::Config;
use crate::error::Result;

pub fn run(command: ConfigCommands) -> Result<()> {
    match command {
        ConfigCommands::Show => {
            let config = Config::load()?;
            println!("📝 Current Configuration:");
            println!("========================");
            println!(
                "Data Directory: {}",
                config
                    .data_directory
                    .as_deref()
                    .unwrap_or("Default (home directory)")
            );
            println!(
                "Default Currency: {}",
                config.default_currency.as_deref().unwrap_or("USD")
            );
            println!(
                "Date Format: {}",
                config.date_format.as_deref().unwrap_or("YYYY-MM-DD")
            );
            println!(
                "Show Dividends: {}",
                config
                    .show_dividends
                    .map(|b| if b { "Yes" } else { "No" })
                    .unwrap_or("Yes")
            );
            println!(
                "Color Output: {}",
                config
                    .color_output
                    .map(|b| if b { "Yes" } else { "No" })
                    .unwrap_or("Yes")
            );
        }
        ConfigCommands::Set { key, value } => {
            let mut config = Config::load()?;
            match key.to_lowercase().as_str() {
                "data-directory" | "data_directory" => {
                    let v = value.clone();
                    config.data_directory = Some(value);
                    println!("✓ Set data directory to: {}", v);
                }
                "currency" => {
                    let v = value.clone();
                    config.default_currency = Some(value);
                    println!("✓ Set default currency to: {}", v);
                }
                "date-format" | "date_format" => {
                    let v = value.clone();
                    config.date_format = Some(value);
                    println!("✓ Set date format to: {}", v);
                }
                "show-dividends" | "show_dividends" => {
                    config.show_dividends = Some(value.parse::<bool>()?);
                    println!("✓ Set show dividends to: {}", value);
                }
                "color-output" | "color_output" => {
                    config.color_output = Some(value.parse::<bool>()?);
                    println!("✓ Set color output to: {}", value);
                }
                other => {
                    return Err(anyhow::anyhow!(
                        "Unknown configuration key: {}. Valid keys are: data-directory, currency, date-format, show-dividends, color-output",
                        other
                    ));
                }
            }
            config.save()?;
            println!("✓ Configuration saved successfully");
        }
        ConfigCommands::Reset => {
            Config::new().save()?;
            println!("✓ Configuration reset to defaults");
        }
    }
    Ok(())
}
