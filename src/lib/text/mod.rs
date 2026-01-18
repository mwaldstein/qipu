//! Text processing utilities for tokenization and ranking

use crate::lib::index::types::Index;
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Common English stop words to filter out during tokenization
static STOP_WORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();

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

/// Calculate BM25 score for a piece of text against a set of query terms
pub fn calculate_bm25(
    query_terms: &[String],
    text: &str,
    index: &Index,
    field_doc_len: Option<usize>,
) -> f64 {
    if text.is_empty() || query_terms.is_empty() {
        return 0.0;
    }

    let terms = tokenize(text);
    let doc_len = field_doc_len.unwrap_or(terms.len());
    if doc_len == 0 {
        return 0.0;
    }

    let mut term_freqs = HashMap::new();
    for term in terms {
        *term_freqs.entry(term).or_insert(0) += 1;
    }

    let total_docs = index.total_docs;
    if total_docs == 0 {
        return 0.0;
    }

    // Average document length from index stats (based on bodies)
    let avgdl = (index.total_len as f64 / total_docs as f64).max(1.0);

    let k1 = 1.2;
    let b = 0.75;
    let mut score = 0.0;

    for query_term in query_terms {
        if let Some(&f) = term_freqs.get(query_term) {
            // Document frequency from index (based on bodies)
            let df = *index.term_df.get(query_term).unwrap_or(&1);

            // Lucene-style BM25 IDF: ln(1 + (N - n + 0.5) / (n + 0.5))
            let idf = ((total_docs as f64 - df as f64 + 0.5) / (df as f64 + 0.5) + 1.0).ln();

            let numerator = f as f64 * (k1 + 1.0);
            let denominator = f as f64 + k1 * (1.0 - b + b * (doc_len as f64 / avgdl));

            score += idf * numerator / denominator;
        }
    }

    score
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
}
