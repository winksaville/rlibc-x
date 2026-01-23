//! Output formatting and filtering for func-analysis

use crate::types::{AnalysisResult, FunctionInfo};
use anyhow::Result;
use clap::ValueEnum;

/// Output format options
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

/// Sort options for function list
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum SortBy {
    Name,
    Size,
    Refs,
}

/// Filter and sort functions based on criteria
pub fn filter_and_sort(min_refs: usize, sort_by: SortBy, functions: &mut Vec<FunctionInfo>) {
    // Filter by min refs
    functions.retain(|f| f.references >= min_refs);

    // Sort
    match sort_by {
        SortBy::Name => functions.sort_by(|a, b| a.name.cmp(&b.name)),
        SortBy::Size => functions.sort_by(|a, b| b.size.cmp(&a.size)),
        SortBy::Refs => functions.sort_by(|a, b| b.references.cmp(&a.references)),
    }
}

/// Output analysis result in the specified format
pub fn output_result(format: OutputFormat, result: &AnalysisResult) -> Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(result)?);
        }
        OutputFormat::Csv => {
            println!("name,size,address,references,source");
            for f in &result.functions {
                println!(
                    "{},{},{:#x},{},{}",
                    f.name,
                    f.size,
                    f.address,
                    f.references,
                    f.source.as_deref().unwrap_or("")
                );
            }
        }
        OutputFormat::Table => {
            let coverage = if result.text_section_size > 0 {
                (result.total_code_size as f64 / result.text_section_size as f64) * 100.0
            } else {
                0.0
            };

            println!("Binary: {}", result.binary_path);
            println!(
                "Type: {}",
                if result.is_dynamic {
                    "dynamic"
                } else {
                    "static"
                }
            );
            println!("Functions: {}", result.total_functions);
            println!("Functions size: {} bytes", result.total_code_size);
            println!(".text size: {} bytes", result.text_section_size);
            println!("Coverage: {:.1}%", coverage);
            println!();
            println!(
                "{:<40} {:>10} {:>12} {:>8}",
                "FUNCTION", "SIZE", "ADDRESS", "REFS"
            );
            println!("{}", "-".repeat(74));

            for f in &result.functions {
                println!(
                    "{:<40} {:>10} {:#12x} {:>8}",
                    truncate(&f.name, 40),
                    f.size,
                    f.address,
                    f.references
                );
            }
        }
    }

    Ok(())
}

/// Truncate a string to max_len, adding "..." if truncated
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_short_string() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn test_truncate_exact_length() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn test_truncate_long_string() {
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_filter_and_sort_by_refs() {
        let mut funcs = vec![
            FunctionInfo {
                name: "foo".to_string(),
                size: 100,
                address: 0x1000,
                references: 5,
                source: None,
            },
            FunctionInfo {
                name: "bar".to_string(),
                size: 200,
                address: 0x2000,
                references: 10,
                source: None,
            },
            FunctionInfo {
                name: "baz".to_string(),
                size: 50,
                address: 0x3000,
                references: 2,
                source: None,
            },
        ];

        filter_and_sort(3, SortBy::Refs, &mut funcs);

        assert_eq!(funcs.len(), 2); // baz filtered out (refs < 3)
        assert_eq!(funcs[0].name, "bar"); // highest refs first
        assert_eq!(funcs[1].name, "foo");
    }

    #[test]
    fn test_filter_and_sort_by_size() {
        let mut funcs = vec![
            FunctionInfo {
                name: "small".to_string(),
                size: 50,
                address: 0x1000,
                references: 1,
                source: None,
            },
            FunctionInfo {
                name: "large".to_string(),
                size: 200,
                address: 0x2000,
                references: 1,
                source: None,
            },
        ];

        filter_and_sort(0, SortBy::Size, &mut funcs);

        assert_eq!(funcs[0].name, "large"); // largest first
        assert_eq!(funcs[1].name, "small");
    }

    #[test]
    fn test_filter_and_sort_by_name() {
        let mut funcs = vec![
            FunctionInfo {
                name: "zebra".to_string(),
                size: 100,
                address: 0x1000,
                references: 1,
                source: None,
            },
            FunctionInfo {
                name: "alpha".to_string(),
                size: 100,
                address: 0x2000,
                references: 1,
                source: None,
            },
        ];

        filter_and_sort(0, SortBy::Name, &mut funcs);

        assert_eq!(funcs[0].name, "alpha"); // alphabetical
        assert_eq!(funcs[1].name, "zebra");
    }
}
