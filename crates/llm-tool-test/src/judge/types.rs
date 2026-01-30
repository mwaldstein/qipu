use serde::{Deserialize, Serialize};

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
