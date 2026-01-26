use anyhow::Result;

use crate::binary::{binary_size, strip_binary};
use crate::build::build_crate;
use crate::run::run_binary;

/// Result of building and running a spec
struct SpecResult {
    size: u64,
    exit_code: i32,
}

/// Compare two specs for a crate
pub fn compare_specs(crate_name: &str, spec_a: &str, spec_b: &str, release: bool) -> Result<()> {
    println!("Comparing {} builds:\n", crate_name);

    // Build, strip, and run spec A
    let result_a = build_and_run(crate_name, spec_a, release)?;

    println!();

    // Build, strip, and run spec B
    let result_b = build_and_run(crate_name, spec_b, release)?;

    // Print comparison
    println!();
    print_comparison(&result_a, &result_b, spec_a, spec_b);

    Ok(())
}

fn build_and_run(crate_name: &str, spec: &str, release: bool) -> Result<SpecResult> {
    println!("  {}:", spec);

    // Build
    let build_result = build_crate(crate_name, Some(spec), release)?;

    // Strip
    strip_binary(&build_result.binary_path)?;

    // Get size
    let size = binary_size(&build_result.binary_path)?;
    println!("    size: {} bytes", format_size(size));

    // Run
    let exit_code = run_binary(&build_result.binary_path)?;
    println!("    exit: {}", exit_code);

    Ok(SpecResult { size, exit_code })
}

fn print_comparison(a: &SpecResult, b: &SpecResult, spec_a: &str, spec_b: &str) {
    // Size comparison
    let size_diff = (a.size as i64 - b.size as i64).unsigned_abs();
    let larger_size = a.size.max(b.size);
    let size_pct = if larger_size > 0 {
        (size_diff as f64 / larger_size as f64) * 100.0
    } else {
        0.0
    };

    let size_str = if a.size < b.size {
        format!(
            "{} is {} smaller ({:.1}%)",
            spec_a,
            format_size(size_diff),
            size_pct
        )
    } else if b.size < a.size {
        format!(
            "{} is {} smaller ({:.1}%)",
            spec_b,
            format_size(size_diff),
            size_pct
        )
    } else {
        "identical".to_string()
    };

    println!("  Size: {}", size_str);

    // Behavior comparison
    if a.exit_code == b.exit_code {
        println!("  Behavior: identical (exit {})", a.exit_code);
    } else {
        println!(
            "  Behavior: DIFFERENT (exit {} vs {}) ⚠️",
            a.exit_code, b.exit_code
        );
    }
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
