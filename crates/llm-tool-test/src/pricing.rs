#[derive(Debug, PartialEq)]
pub struct ModelPricing {
    pub input_cost_per_1k_tokens: f64,
    pub output_cost_per_1k_tokens: f64,
}

pub fn get_model_pricing(model: &str) -> Option<ModelPricing> {
    let model_lower = model.to_lowercase();

    let pricing = match model_lower.as_str() {
        m if m.contains("claude-3-5-sonnet") || m.contains("sonnet") => ModelPricing {
            input_cost_per_1k_tokens: 3.0,
            output_cost_per_1k_tokens: 15.0,
        },
        m if m.contains("claude-3-5-haiku") || m.contains("haiku") => ModelPricing {
            input_cost_per_1k_tokens: 0.8,
            output_cost_per_1k_tokens: 4.0,
        },
        m if m.contains("claude-3-opus") || m.contains("opus") => ModelPricing {
            input_cost_per_1k_tokens: 15.0,
            output_cost_per_1k_tokens: 75.0,
        },
        m if m.contains("claude-3") => ModelPricing {
            input_cost_per_1k_tokens: 3.0,
            output_cost_per_1k_tokens: 15.0,
        },
        m if m.contains("claude") => ModelPricing {
            input_cost_per_1k_tokens: 3.0,
            output_cost_per_1k_tokens: 15.0,
        },

        m if m.contains("gpt-4o") => ModelPricing {
            input_cost_per_1k_tokens: 2.5,
            output_cost_per_1k_tokens: 10.0,
        },
        m if m.contains("gpt-4-turbo") || m.contains("gpt-4-turbo-preview") => ModelPricing {
            input_cost_per_1k_tokens: 10.0,
            output_cost_per_1k_tokens: 30.0,
        },
        m if m.contains("gpt-4") => ModelPricing {
            input_cost_per_1k_tokens: 30.0,
            output_cost_per_1k_tokens: 60.0,
        },
        m if m.contains("gpt-3.5-turbo") => ModelPricing {
            input_cost_per_1k_tokens: 0.5,
            output_cost_per_1k_tokens: 1.5,
        },
        m if m.contains("gpt-3.5") => ModelPricing {
            input_cost_per_1k_tokens: 0.5,
            output_cost_per_1k_tokens: 1.5,
        },

        "smart" => ModelPricing {
            input_cost_per_1k_tokens: 3.0,
            output_cost_per_1k_tokens: 15.0,
        },
        "rush" => ModelPricing {
            input_cost_per_1k_tokens: 0.8,
            output_cost_per_1k_tokens: 4.0,
        },
        "free" => ModelPricing {
            input_cost_per_1k_tokens: 0.0,
            output_cost_per_1k_tokens: 0.0,
        },

        _ => return None,
    };

    Some(pricing)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_model_pricing_claude_sonnet() {
        let pricing = get_model_pricing("claude-3-5-sonnet-20241022");
        assert!(pricing.is_some());
        let p = pricing.unwrap();
        assert_eq!(p.input_cost_per_1k_tokens, 3.0);
        assert_eq!(p.output_cost_per_1k_tokens, 15.0);
    }

    #[test]
    fn test_get_model_pricing_claude_haiku() {
        let pricing = get_model_pricing("claude-3-5-haiku-20241022");
        assert!(pricing.is_some());
        let p = pricing.unwrap();
        assert_eq!(p.input_cost_per_1k_tokens, 0.8);
        assert_eq!(p.output_cost_per_1k_tokens, 4.0);
    }

    #[test]
    fn test_get_model_pricing_gpt4o() {
        let pricing = get_model_pricing("gpt-4o");
        assert!(pricing.is_some());
        let p = pricing.unwrap();
        assert_eq!(p.input_cost_per_1k_tokens, 2.5);
        assert_eq!(p.output_cost_per_1k_tokens, 10.0);
    }

    #[test]
    fn test_get_model_pricing_unknown_model() {
        let pricing = get_model_pricing("unknown-model");
        assert!(pricing.is_none());
    }

    #[test]
    fn test_get_model_pricing_case_insensitive() {
        let pricing1 = get_model_pricing("GPT-4O");
        let pricing2 = get_model_pricing("gpt-4o");
        assert_eq!(pricing1, pricing2);
    }
}
