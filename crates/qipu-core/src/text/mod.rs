//! Text processing utilities for tokenization and ranking

use rust_stemmers::{Algorithm, Stemmer};
use std::collections::HashSet;
use std::sync::OnceLock;

/// Common English stop words to filter out during tokenization
static STOP_WORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();

/// Porter stemmer for English text
static STEMMER: OnceLock<Stemmer> = OnceLock::new();

fn get_stop_words() -> &'static HashSet<&'static str> {
    STOP_WORDS.get_or_init(|| {
        [
            "a", "an", "and", "are", "as", "at", "be", "but", "by", "for", "if", "in", "into",
            "is", "it", "no", "not", "of", "on", "or", "such", "that", "the", "their", "then",
            "there", "these", "they", "this", "to", "was", "will", "with",
        ]
        .iter()
        .copied()
        .collect()
    })
}

fn get_stemmer() -> &'static Stemmer {
    STEMMER.get_or_init(|| Stemmer::create(Algorithm::English))
}

/// Simple word-based tokenizer splitting on non-alphanumeric characters with stop word removal
pub fn tokenize(text: &str) -> Vec<String> {
    let stop_words = get_stop_words();
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .filter(|s| !stop_words.contains(s))
        .map(|s| s.to_string())
        .collect()
}

/// Tokenize text with optional Porter stemming
///
/// When `stem` is true, applies Porter stemming to match words like "graph" and "graphs"
/// This improves similarity calculation for the similarity engine
pub fn tokenize_with_stemming(text: &str, stem: bool) -> Vec<String> {
    let tokens = tokenize(text);
    if !stem {
        return tokens;
    }

    let stemmer = get_stemmer();
    tokens.iter().map(|t| stemmer.stem(t).to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_basic() {
        let text = "Hello world! This is a test.";
        let tokens = tokenize(text);
        // Should filter out "a", "is", "this"
        assert_eq!(tokens, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_tokenize_removes_stop_words() {
        let text = "the quick brown fox";
        let tokens = tokenize(text);
        // "the" is a stop word
        assert_eq!(tokens, vec!["quick", "brown", "fox"]);
    }

    #[test]
    fn test_tokenize_keeps_content_words() {
        let text = "zettelkasten ontology knowledge graph";
        let tokens = tokenize(text);
        assert_eq!(
            tokens,
            vec!["zettelkasten", "ontology", "knowledge", "graph"]
        );
    }

    #[test]
    fn test_tokenize_mixed_stop_and_content() {
        let text = "This is the way to build a system";
        let tokens = tokenize(text);
        // Filters: this, is, the, to, a
        assert_eq!(tokens, vec!["way", "build", "system"]);
    }

    #[test]
    fn test_tokenize_empty_after_stop_words() {
        let text = "the a an and or";
        let tokens = tokenize(text);
        assert_eq!(tokens, Vec::<String>::new());
    }

    #[test]
    fn test_tokenize_preserves_capitalization_in_lowercase() {
        let text = "Graph THEORY and Networks";
        let tokens = tokenize(text);
        // "and" is filtered
        assert_eq!(tokens, vec!["graph", "theory", "networks"]);
    }

    #[test]
    fn test_tokenize_with_stemming_disabled() {
        let text = "Graph graphs network networks";
        let tokens = tokenize_with_stemming(text, false);
        assert_eq!(tokens, vec!["graph", "graphs", "network", "networks"]);
    }

    #[test]
    fn test_tokenize_with_stemming_enabled() {
        let text = "Graph graphs network networks";
        let tokens = tokenize_with_stemming(text, true);
        // Porter stemming should reduce plurals to singular forms
        assert_eq!(tokens, vec!["graph", "graph", "network", "network"]);
    }

    #[test]
    fn test_stemming_matches_similar_words() {
        let text1 = "The knowledge graph system";
        let text2 = "Graphing knowledge networks";

        let tokens1 = tokenize_with_stemming(text1, true);
        let tokens2 = tokenize_with_stemming(text2, true);

        // Both should contain "graph" due to stemming
        assert!(tokens1.contains(&"graph".to_string()));
        assert!(tokens2.contains(&"graph".to_string()));
    }
}
