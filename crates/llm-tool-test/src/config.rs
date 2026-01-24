use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricingConfig {
    pub input_cost_per_1k_tokens: f64,
    pub output_cost_per_1k_tokens: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub models: std::collections::HashMap<String, ModelPricingConfig>,
}

impl Config {
    pub fn load_or_default() -> Self {
        let config_path = Path::new("llm-tool-test-config.toml");

        if config_path.exists() {
            match Self::load(config_path) {
                Ok(config) => return config,
                Err(e) => {
                    eprintln!("Warning: Failed to load config file: {}", e);
                    eprintln!("Using default pricing configuration");
                }
            }
        }

        Self::with_defaults()
    }

    pub fn load(path: &Path) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: Config =
            toml::from_str(&content).map_err(|e| format!("Failed to parse config file: {}", e))?;

        Ok(config)
    }

    pub fn with_defaults() -> Self {
        let mut models = std::collections::HashMap::new();

        models.insert(
            "claude-3-5-sonnet".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 3.0,
                output_cost_per_1k_tokens: 15.0,
            },
        );

        models.insert(
            "claude-3-5-haiku".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 0.8,
                output_cost_per_1k_tokens: 4.0,
            },
        );

        models.insert(
            "claude-3-opus".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 15.0,
                output_cost_per_1k_tokens: 75.0,
            },
        );

        models.insert(
            "claude-3".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 3.0,
                output_cost_per_1k_tokens: 15.0,
            },
        );

        models.insert(
            "claude".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 3.0,
                output_cost_per_1k_tokens: 15.0,
            },
        );

        models.insert(
            "gpt-4o".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 2.5,
                output_cost_per_1k_tokens: 10.0,
            },
        );

        models.insert(
            "gpt-4-turbo".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 10.0,
                output_cost_per_1k_tokens: 30.0,
            },
        );

        models.insert(
            "gpt-4".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 30.0,
                output_cost_per_1k_tokens: 60.0,
            },
        );

        models.insert(
            "gpt-3.5-turbo".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 0.5,
                output_cost_per_1k_tokens: 1.5,
            },
        );

        models.insert(
            "gpt-3.5".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 0.5,
                output_cost_per_1k_tokens: 1.5,
            },
        );

        models.insert(
            "smart".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 3.0,
                output_cost_per_1k_tokens: 15.0,
            },
        );

        models.insert(
            "rush".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 0.8,
                output_cost_per_1k_tokens: 4.0,
            },
        );

        models.insert(
            "free".to_string(),
            ModelPricingConfig {
                input_cost_per_1k_tokens: 0.0,
                output_cost_per_1k_tokens: 0.0,
            },
        );

        Config { models }
    }

    pub fn get_model_pricing(&self, model: &str) -> Option<crate::pricing::ModelPricing> {
        let model_lower = model.to_lowercase();

        let mut keys: Vec<_> = self.models.keys().collect();
        keys.sort_by(|a, b| b.len().cmp(&a.len()));

        for key in keys {
            if model_lower.contains(key) {
                let pricing = &self.models[key];
                return Some(crate::pricing::ModelPricing {
                    input_cost_per_1k_tokens: pricing.input_cost_per_1k_tokens,
                    output_cost_per_1k_tokens: pricing.output_cost_per_1k_tokens,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_defaults() {
        let config = Config::with_defaults();
        assert!(!config.models.is_empty());
        assert!(config.models.contains_key("gpt-4o"));
        assert!(config.models.contains_key("claude-3-5-sonnet"));
    }

    #[test]
    fn test_get_model_pricing() {
        let config = Config::with_defaults();

        let pricing = config.get_model_pricing("gpt-4o");
        assert!(pricing.is_some());
        let p = pricing.unwrap();
        assert_eq!(p.input_cost_per_1k_tokens, 2.5);
        assert_eq!(p.output_cost_per_1k_tokens, 10.0);
    }

    #[test]
    fn test_get_model_pricing_case_insensitive() {
        let config = Config::with_defaults();

        let pricing1 = config.get_model_pricing("GPT-4O");
        let pricing2 = config.get_model_pricing("gpt-4o");
        assert_eq!(pricing1, pricing2);
    }

    #[test]
    fn test_get_model_pricing_unknown() {
        let config = Config::with_defaults();

        let pricing = config.get_model_pricing("unknown-model");
        assert!(pricing.is_none());
    }

    #[test]
    fn test_load_and_save() {
        let config = Config::with_defaults();

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test-config.toml");

        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&path, content).unwrap();

        let loaded = Config::load(&path).unwrap();
        assert_eq!(loaded.models.len(), config.models.len());
    }
}
