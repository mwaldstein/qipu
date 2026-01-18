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
}
