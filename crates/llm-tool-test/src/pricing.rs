#![allow(dead_code)]

#[derive(Debug, PartialEq)]
pub struct ModelPricing {
    pub input_cost_per_1k_tokens: f64,
    pub output_cost_per_1k_tokens: f64,
}

pub fn get_model_pricing(model: &str) -> Option<ModelPricing> {
    crate::config::Config::load_or_default().get_model_pricing(model)
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

    #[test]
    fn test_get_model_pricing_amp_smart() {
        let pricing = get_model_pricing("smart");
        assert!(pricing.is_some());
        let p = pricing.unwrap();
        assert_eq!(p.input_cost_per_1k_tokens, 3.0);
        assert_eq!(p.output_cost_per_1k_tokens, 15.0);
    }

    #[test]
    fn test_get_model_pricing_amp_free() {
        let pricing = get_model_pricing("free");
        assert!(pricing.is_some());
        let p = pricing.unwrap();
        assert_eq!(p.input_cost_per_1k_tokens, 0.0);
        assert_eq!(p.output_cost_per_1k_tokens, 0.0);
    }
}
