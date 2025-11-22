// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                   LICENSE KEY GENERATOR TOOL                              ║
// ║                   Copyright (c) 2025 S1BGr0uP                             ║
// ║                        All Rights Reserved                                ║
// ║  CONFIDENTIAL: This tool is for S1BGr0uP internal use only.               ║
// ║  DO NOT DISTRIBUTE. Keep this tool and generated keys secure.             ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

use clap::{Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
enum Tier {
    Free,
    Professional,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct License {
    key: String,
    tier: Tier,
    licensee: String,
    issued_at: u64,
    expires_at: Option<u64>,
    hardware_id: Option<String>,
}

#[derive(Parser)]
#[command(name = "license-gen")]
#[command(about = "Int3rceptor License Key Generator (Internal Use Only)")]
struct Cli {
    /// Licensee name or organization
    #[arg(short, long)]
    licensee: String,

    /// License tier
    #[arg(short, long, value_enum)]
    tier: Tier,

    /// Days until expiration (omit for perpetual)
    #[arg(short, long)]
    days: Option<u64>,

    /// Hardware ID for binding (optional)
    #[arg(long)]
    hardware_id: Option<String>,

    /// Output file (default: stdout)
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let expires_at = cli.days.map(|days| now + (days * 86400));

    let license = License {
        key: String::new(), // Will be filled after encoding
        tier: cli.tier,
        licensee: cli.licensee.clone(),
        issued_at: now,
        expires_at,
        hardware_id: cli.hardware_id.clone(),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&license)?;

    // Encode to base64
    #[allow(deprecated)]
    let encoded = base64::encode(json.as_bytes());

    // Display license information
    println!("\n╔═══════════════════════════════════════════════════════════════════════════╗");
    println!("║                   INT3RCEPTOR LICENSE GENERATED                           ║");
    println!("╠═══════════════════════════════════════════════════════════════════════════╣");
    println!("║  Licensee: {:<63} ║", cli.licensee);
    println!("║  Tier: {:<68} ║", format!("{:?}", cli.tier));
    println!(
        "║  Issued: {:<66} ║",
        chrono::DateTime::from_timestamp(now as i64, 0)
            .unwrap()
            .format("%Y-%m-%d %H:%M:%S UTC")
    );

    if let Some(exp) = expires_at {
        println!(
            "║  Expires: {:<65} ║",
            chrono::DateTime::from_timestamp(exp as i64, 0)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S UTC")
        );
    } else {
        println!("║  Expires: PERPETUAL{:<57} ║", "");
    }

    if let Some(hw_id) = &cli.hardware_id {
        println!("║  Hardware ID: {:<59} ║", hw_id);
    }

    println!("╚═══════════════════════════════════════════════════════════════════════════╝\n");

    println!("LICENSE KEY:");
    println!("{}\n", encoded);

    println!("USAGE:");
    println!("  export INTERCEPTOR_LICENSE_KEY=\"{}\"", encoded);
    println!("  # OR");
    println!("  echo \"{}\" > license.key\n", encoded);

    // Save to file if specified
    if let Some(output_path) = cli.output {
        std::fs::write(&output_path, encoded)?;
        println!("✅ License key saved to: {}", output_path);
    }

    Ok(())
}
