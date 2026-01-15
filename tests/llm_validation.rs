mod llm;

use llm::{OpenCodeAdapter, StoreValidation, ToolAdapter, ValidationConfig, ValidationRunner};
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_config_default() {
        let config = ValidationConfig::default();
        assert_eq!(config.tool, "opencode");
        assert_eq!(config.transcript_base, PathBuf::from("tests/transcripts"));
        assert!(config.keep_transcripts);
        assert_eq!(config.timeout_secs, 300);
    }

    #[test]
    fn test_store_validation_empty() {
        let validation = StoreValidation::empty();
        assert_eq!(validation.note_count, 0);
        assert_eq!(validation.link_count, 0);
        assert!(!validation.has_structure);
        assert!(!validation.captured_task);
        assert!(!validation.is_valid());
    }

    #[test]
    fn test_store_validation_validity() {
        // Test case where task was captured but no structure
        let validation1 = StoreValidation {
            note_count: 2,
            link_count: 0,
            has_structure: false,
            captured_task: true,
            details: vec!["No links".to_string()],
        };
        assert!(!validation1.is_valid()); // No links or structure

        // Test case where task was captured with links
        let validation2 = StoreValidation {
            note_count: 2,
            link_count: 1,
            has_structure: false,
            captured_task: true,
            details: vec!["Has links".to_string()],
        };
        assert!(validation2.is_valid()); // Has links

        // Test case with structure but no links
        let validation3 = StoreValidation {
            note_count: 2,
            link_count: 0,
            has_structure: true,
            captured_task: true,
            details: vec!["Has structure".to_string()],
        };
        assert!(validation3.is_valid()); // Has structure
    }

    #[test]
    fn test_opencode_adapter() {
        let adapter = OpenCodeAdapter::new();
        assert_eq!(adapter.name(), "opencode");
        // The availability check will depend on whether opencode is installed
        // We just test that the method doesn't panic
        let _is_available = adapter.is_available();
    }

    #[test]
    fn test_validation_runner_creation() {
        let config = ValidationConfig::default();
        let runner = ValidationRunner::new(config.clone());
        // We can't easily test the full validation without a proper environment,
        // but we can test that the runner is created correctly
        assert_eq!(runner.config.tool, config.tool);
        assert_eq!(runner.config.transcript_base, config.transcript_base);
    }

    #[test]
    fn test_full_validation_simulation() {
        // This test runs the full validation framework with simulated task execution
        let config = ValidationConfig {
            tool: "opencode".to_string(),
            transcript_base: PathBuf::from("tests/transcripts"),
            keep_transcripts: true,
            timeout_secs: 60,
        };

        let runner = ValidationRunner::new(config);

        let task_prompt = "Create some research notes about machine learning and connect them with meaningful links";

        let result = runner
            .run_validation(task_prompt)
            .expect("Validation should complete without errors");

        // Check that validation completed
        assert!(
            result.transcript_path.is_some(),
            "Transcript path should be set"
        );

        // The simulation should always pass since it creates notes and links
        assert!(result.passed, "Simulated validation should pass");
        assert!(
            result.message.contains("successfully created"),
            "Success message should be present"
        );

        // Check that the store validation reflects the simulated data
        assert!(
            result.store_validation.note_count > 0,
            "Should have created some notes"
        );
        assert!(
            result.store_validation.link_count > 0,
            "Should have created some links"
        );
        assert!(
            result.store_validation.captured_task,
            "Task should be captured"
        );
        assert!(result.store_validation.is_valid(), "Store should be valid");

        // Verify transcript files were created
        let transcript_path = result.transcript_path.unwrap();
        assert!(
            transcript_path.exists(),
            "Transcript directory should exist"
        );
        assert!(
            transcript_path.join("task_prompt.txt").exists(),
            "Task prompt file should exist"
        );
        assert!(
            transcript_path.join("commands.log").exists(),
            "Commands log should exist"
        );
        assert!(
            transcript_path.join("session_summary.txt").exists(),
            "Session summary should exist"
        );
    }
}
