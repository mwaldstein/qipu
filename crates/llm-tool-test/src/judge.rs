use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rubric {
    pub criteria: Vec<Criterion>,
    pub output: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Criterion {
    pub id: String,
    pub weight: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFormat {
    pub format: String,
    pub require_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgeResponse {
    pub scores: std::collections::HashMap<String, f64>,
    pub weighted_score: f64,
    pub confidence: f64,
    pub issues: Vec<String>,
    pub highlights: Vec<String>,
}

pub fn load_rubric(path: &Path) -> Result<Rubric> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read rubric file: {}", path.display()))?;
    let rubric: Rubric = serde_yaml::from_str(&content)
        .with_context(|| format!("Failed to parse rubric YAML: {}", path.display()))?;

    let total_weight: f64 = rubric.criteria.iter().map(|c| c.weight).sum();
    if (total_weight - 1.0).abs() > 0.01 {
        anyhow::bail!(
            "Rubric criterion weights must sum to 1.0, got {}",
            total_weight
        );
    }

    Ok(rubric)
}

pub async fn run_judge(
    model: &str,
    transcript_summary: &str,
    store_export: &str,
    task_description: &str,
    rubric: &Rubric,
) -> Result<JudgeResponse> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .or_else(|_| std::env::var("LLM_TOOL_TEST_API_KEY"))
        .context("OPENAI_API_KEY or LLM_TOOL_TEST_API_KEY environment variable must be set")?;

    let client = reqwest::Client::new();

    let prompt = build_judge_prompt(transcript_summary, store_export, task_description, rubric);

    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are an expert evaluator. Analyze the provided transcript and store state against the given rubric. Return your evaluation as JSON only."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "response_format": { "type": "json_object" },
        "temperature": 0.3,
        "max_tokens": 2000,
    });

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .context("Failed to call OpenAI API")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("OpenAI API request failed: {} - {}", status, error_text);
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse OpenAI API response")?;

    let content = response_json
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .context("Invalid OpenAI API response format")?;

    let judge_response: JudgeResponse = serde_json::from_str(content)
        .with_context(|| format!("Failed to parse judge response JSON: {}", content))?;

    Ok(judge_response)
}

fn build_judge_prompt(
    transcript_summary: &str,
    store_export: &str,
    task_description: &str,
    rubric: &Rubric,
) -> String {
    let criteria_text = rubric
        .criteria
        .iter()
        .map(|c| format!("- {}: {} (weight: {:.2})", c.id, c.description, c.weight))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"Evaluate the following LLM tool interaction.

# Task
{}

# Transcript Summary
{}

# Store State (JSON)
{}

# Evaluation Criteria
{}

# Required Response Format
Return JSON with this exact structure:
{{
  "scores": {{
    "criterion_id": <score_0_to_1>,
    ...
  }},
  "weighted_score": <weighted_average_0_to_1>,
  "confidence": <confidence_0_to_1>,
  "issues": ["issue1", "issue2", ...],
  "highlights": ["good_practice1", "good_practice2", ...]
}}

Provide JSON only, no additional text."#,
        task_description, transcript_summary, store_export, criteria_text
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_build_judge_prompt() {
        let rubric = Rubric {
            criteria: vec![Criterion {
                id: "test_criterion".to_string(),
                weight: 1.0,
                description: "Test description".to_string(),
            }],
            output: OutputFormat {
                format: "json".to_string(),
                require_fields: vec!["scores".to_string()],
            },
        };

        let prompt = build_judge_prompt("Test task", "Test transcript", "{}", &rubric);

        assert!(prompt.contains("Test task"));
        assert!(prompt.contains("Test transcript"));
        assert!(prompt.contains("test_criterion"));
    }

    #[test]
    fn test_load_rubric_weights_sum_to_one() {
        let rubric_yaml = r#"
criteria:
  - id: relevance
    weight: 0.35
    description: "Notes directly address the task prompt"
  - id: coherence
    weight: 0.35
    description: "Notes are logically connected"
  - id: granularity
    weight: 0.30
    description: "Notes are appropriately scoped"
output:
  format: json
  require_fields:
    - scores
    - weighted_score
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let rubric_path = temp_dir.path().join("test_rubric.yaml");
        std::fs::write(&rubric_path, rubric_yaml).unwrap();

        let rubric = load_rubric(&rubric_path).unwrap();
        assert_eq!(rubric.criteria.len(), 3);
        assert_eq!(rubric.criteria[0].id, "relevance");
        assert_eq!(rubric.criteria[0].weight, 0.35);
    }

    #[test]
    fn test_load_rubric_weights_sum_error() {
        let rubric_yaml = r#"
criteria:
  - id: criterion1
    weight: 0.5
    description: "First criterion"
  - id: criterion2
    weight: 0.4
    description: "Second criterion"
output:
  format: json
  require_fields:
    - scores
"#;
        let temp_dir = tempfile::tempdir().unwrap();
        let rubric_path = temp_dir.path().join("bad_rubric.yaml");
        std::fs::write(&rubric_path, rubric_yaml).unwrap();

        let result = load_rubric(&rubric_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must sum to 1.0"));
    }

    #[tokio::test]
    async fn test_run_judge_missing_api_key() {
        let rubric = Rubric {
            criteria: vec![Criterion {
                id: "test".to_string(),
                weight: 1.0,
                description: "Test".to_string(),
            }],
            output: OutputFormat {
                format: "json".to_string(),
                require_fields: vec![],
            },
        };

        std::env::remove_var("OPENAI_API_KEY");
        std::env::remove_var("LLM_TOOL_TEST_API_KEY");

        let result = run_judge("gpt-4o-mini", "transcript", "{}", "task", &rubric).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("environment variable must be set"));
    }

    #[tokio::test]
    async fn test_run_judge_success() {
        let mock_server = MockServer::start().await;
        let api_url = mock_server.uri();

        let rubric = Rubric {
            criteria: vec![Criterion {
                id: "test_criterion".to_string(),
                weight: 1.0,
                description: "Test criterion".to_string(),
            }],
            output: OutputFormat {
                format: "json".to_string(),
                require_fields: vec![],
            },
        };

        let mock_response = serde_json::json!({
            "choices": [{
                "message": {
                    "content": r#"{
                        "scores": {"test_criterion": 0.85},
                        "weighted_score": 0.85,
                        "confidence": 0.9,
                        "issues": [],
                        "highlights": ["Good execution"]
                    }"#
                }
            }]
        });

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/v1/chat/completions"))
            .and(matchers::header("authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let result = run_judge_with_client(
            "gpt-4o-mini",
            "transcript summary",
            r#"{"notes": []}"#,
            "task description",
            &rubric,
            &api_url,
            "test-key",
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.scores.get("test_criterion"), Some(&0.85));
        assert_eq!(response.weighted_score, 0.85);
        assert_eq!(response.confidence, 0.9);
    }

    #[tokio::test]
    async fn test_run_judge_api_error() {
        let mock_server = MockServer::start().await;
        let api_url = mock_server.uri();

        let rubric = Rubric {
            criteria: vec![Criterion {
                id: "test".to_string(),
                weight: 1.0,
                description: "Test".to_string(),
            }],
            output: OutputFormat {
                format: "json".to_string(),
                require_fields: vec![],
            },
        };

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
                "error": {"message": "Rate limit exceeded"}
            })))
            .mount(&mock_server)
            .await;

        let result = run_judge_with_client(
            "gpt-4o-mini",
            "transcript",
            "{}",
            "task",
            &rubric,
            &api_url,
            "test-key",
        )
        .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("OpenAI API request failed"));
        assert!(err_msg.contains("429"));
    }

    #[tokio::test]
    async fn test_run_judge_invalid_response_format() {
        let mock_server = MockServer::start().await;
        let api_url = mock_server.uri();

        let rubric = Rubric {
            criteria: vec![Criterion {
                id: "test".to_string(),
                weight: 1.0,
                description: "Test".to_string(),
            }],
            output: OutputFormat {
                format: "json".to_string(),
                require_fields: vec![],
            },
        };

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{"message": {"content": "not valid json"}}]
            })))
            .mount(&mock_server)
            .await;

        let result = run_judge_with_client(
            "gpt-4o-mini",
            "transcript",
            "{}",
            "task",
            &rubric,
            &api_url,
            "test-key",
        )
        .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to parse judge response JSON"));
    }

    #[tokio::test]
    async fn test_run_judge_missing_content_field() {
        let mock_server = MockServer::start().await;
        let api_url = mock_server.uri();

        let rubric = Rubric {
            criteria: vec![Criterion {
                id: "test".to_string(),
                weight: 1.0,
                description: "Test".to_string(),
            }],
            output: OutputFormat {
                format: "json".to_string(),
                require_fields: vec![],
            },
        };

        Mock::given(matchers::method("POST"))
            .and(matchers::path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "choices": [{"message": {}}]
            })))
            .mount(&mock_server)
            .await;

        let result = run_judge_with_client(
            "gpt-4o-mini",
            "transcript",
            "{}",
            "task",
            &rubric,
            &api_url,
            "test-key",
        )
        .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Invalid OpenAI API response format"));
    }
}

async fn run_judge_with_client(
    model: &str,
    transcript_summary: &str,
    store_export: &str,
    task_description: &str,
    rubric: &Rubric,
    api_base: &str,
    api_key: &str,
) -> Result<JudgeResponse> {
    use reqwest::Client;

    let client = Client::new();
    let prompt = build_judge_prompt(transcript_summary, store_export, task_description, rubric);

    let request_body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "system",
                "content": "You are an expert evaluator. Analyze the provided transcript and store state against the given rubric. Return your evaluation as JSON only."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "response_format": { "type": "json_object" },
        "temperature": 0.3,
        "max_tokens": 2000,
    });

    let response = client
        .post(&format!("{}/v1/chat/completions", api_base))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .context("Failed to call OpenAI API")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("OpenAI API request failed: {} - {}", status, error_text);
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse OpenAI API response")?;

    let content = response_json
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|arr| arr.first())
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .context("Invalid OpenAI API response format")?;

    let judge_response: JudgeResponse = serde_json::from_str(content)
        .with_context(|| format!("Failed to parse judge response JSON: {}", content))?;

    Ok(judge_response)
}
