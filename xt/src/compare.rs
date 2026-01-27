use std::path::Path;

use anyhow::Result;

use crate::binary::{binary_size, strip_binary};
use crate::build::build_crate;
use crate::{print_header, print_hline};

/// Result of building a spec
struct SpecResult {
    name: String,
    size: u64,
}

/// Compare multiple specs for a crate
pub fn compare_specs(
    crate_name: &str,
    spec_paths: &[impl AsRef<Path>],
    release: bool,
    strip: bool,
) -> Result<()> {
    println!(
        "Comparing {} builds{}:\n",
        crate_name,
        if strip { " (stripped)" } else { "" }
    );

    let mut results = Vec::new();

    for spec_path in spec_paths {
        let spec_path = spec_path.as_ref();
        let name = spec_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| spec_path.display().to_string());

        let size = build_spec(crate_name, spec_path, release, strip)?;
        results.push(SpecResult { name, size });
        println!();
    }

    // Sort by size (smallest first)
    results.sort_by_key(|r| r.size);

    print_comparison(&results);

    Ok(())
}

fn build_spec(crate_name: &str, spec_path: &Path, release: bool, strip: bool) -> Result<u64> {
    let spec_str = spec_path.to_string_lossy();
    println!("  {}:", spec_path.file_name().unwrap_or_default().to_string_lossy());

    // Build
    let build_result = build_crate(crate_name, Some(&spec_str), release)?;

    // Optionally strip
    if strip {
        strip_binary(&build_result.binary_path)?;
    }

    // Get size
    let size = binary_size(&build_result.binary_path)?;
    println!("    size: {} bytes", format_size(size));

    Ok(size)
}

fn print_comparison(results: &[SpecResult]) {
    let largest_size = results.iter().map(|r| r.size).max().unwrap_or(0);
    let max_name_len = results.iter().map(|r| r.name.len()).max().unwrap_or(4);

    // Format percent change: show reduction with minus sign, baseline as 0.0%
    let fmt_pct = |size: u64| -> String {
        if largest_size == 0 {
            return "   0.0%".to_string();
        }
        let pct = ((largest_size as f64 - size as f64) / largest_size as f64) * 100.0;
        if pct > 0.0 {
            format!("{:>7.1}%", -pct)
        } else {
            "   0.0%".to_string()
        }
    };

    println!();
    print_header!("COMPARE SUMMARY");
    println!(
        "  {:width$}  {:>10}  {:>8}",
        "Spec",
        "Size",
        "Change",
        width = max_name_len
    );
    for result in results {
        println!(
            "  {:width$}  {:>10}  {}",
            result.name,
            format_size(result.size),
            fmt_pct(result.size),
            width = max_name_len
        );
    }
    print_hline!();
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
