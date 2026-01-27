use anyhow::Result;

use crate::binary::{binary_size, strip_binary};
use crate::build::build_crate;

/// Compare two specs for a crate
pub fn compare_specs(
    crate_name: &str,
    spec_a: &str,
    spec_b: &str,
    release: bool,
    strip: bool,
) -> Result<()> {
    println!("Comparing {} builds{}:\n", crate_name, if strip { " (stripped)" } else { "" });

    // Build and optionally strip spec A
    let size_a = build_spec(crate_name, spec_a, release, strip)?;

    println!();

    // Build and optionally strip spec B
    let size_b = build_spec(crate_name, spec_b, release, strip)?;

    // Print comparison
    print_comparison(size_a, size_b, spec_a, spec_b);

    Ok(())
}

fn build_spec(crate_name: &str, spec: &str, release: bool, strip: bool) -> Result<u64> {
    println!("  {}:", spec);

    // Build
    let build_result = build_crate(crate_name, Some(spec), release)?;

    // Optionally strip
    if strip {
        strip_binary(&build_result.binary_path)?;
    }

    // Get size
    let size = binary_size(&build_result.binary_path)?;
    println!("    size: {} bytes", format_size(size));

    Ok(size)
}

fn print_comparison(size_a: u64, size_b: u64, spec_a: &str, spec_b: &str) {
    let larger_size = size_a.max(size_b);

    // Calculate percent reduction from larger for each spec
    let pct_a = if larger_size > 0 {
        ((larger_size as f64 - size_a as f64) / larger_size as f64) * 100.0
    } else {
        0.0
    };
    let pct_b = if larger_size > 0 {
        ((larger_size as f64 - size_b as f64) / larger_size as f64) * 100.0
    } else {
        0.0
    };

    // Format percent change: show reduction with minus sign, baseline as 0.0%
    let fmt_pct = |pct: f64| -> String {
        if pct > 0.0 {
            format!("{:>7.1}%", -pct)
        } else {
            "   0.0%".to_string()
        }
    };

    let max_name_len = spec_a.len().max(spec_b.len());

    println!();
    println!("========================================");
    println!("          COMPARE SUMMARY");
    println!("========================================");
    println!();
    println!("  {:width$}  {:>10}  {:>8}", "Spec", "Size", "Change", width = max_name_len);
    println!("  {:width$}  {:>10}  {}", spec_a, format_size(size_a), fmt_pct(pct_a), width = max_name_len);
    println!("  {:width$}  {:>10}  {}", spec_b, format_size(size_b), fmt_pct(pct_b), width = max_name_len);
    println!("========================================");
    println!();
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_000_000 {
        format!("{:.1}M", bytes as f64 / 1_000_000.0)
    } else if bytes >= 1_000 {
        format!("{:.1}K", bytes as f64 / 1_000.0)
    } else {
        format!("{}", bytes)
    }
}
