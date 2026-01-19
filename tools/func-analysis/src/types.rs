//! Core types for func-analysis

use serde::Serialize;

/// Information about a single function
#[derive(Debug, Clone, Serialize)]
pub struct FunctionInfo {
    pub name: String,
    pub size: u64,
    pub address: u64,
    pub references: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>, // "local" or "glibc" etc.
}

/// Result of analyzing a binary
#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    pub binary_path: String,
    pub is_dynamic: bool,
    pub total_functions: usize,
    pub total_code_size: u64,
    pub text_section_size: u64,
    pub functions: Vec<FunctionInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_info_serialization() {
        let func = FunctionInfo {
            name: "malloc".to_string(),
            size: 128,
            address: 0x401000,
            references: 5,
            source: Some("local".to_string()),
        };

        let json = serde_json::to_string(&func).unwrap();
        assert!(json.contains("\"name\":\"malloc\""));
        assert!(json.contains("\"size\":128"));
        assert!(json.contains("\"references\":5"));
    }

    #[test]
    fn test_function_info_skips_none_source() {
        let func = FunctionInfo {
            name: "free".to_string(),
            size: 64,
            address: 0x401100,
            references: 3,
            source: None,
        };

        let json = serde_json::to_string(&func).unwrap();
        assert!(!json.contains("source"));
    }

    #[test]
    fn test_analysis_result_serialization() {
        let result = AnalysisResult {
            binary_path: "/path/to/binary".to_string(),
            is_dynamic: false,
            total_functions: 2,
            total_code_size: 192,
            text_section_size: 1024,
            functions: vec![
                FunctionInfo {
                    name: "main".to_string(),
                    size: 100,
                    address: 0x401000,
                    references: 1,
                    source: None,
                },
            ],
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"is_dynamic\":false"));
        assert!(json.contains("\"total_functions\":2"));
        assert!(json.contains("\"text_section_size\":1024"));
    }
}
