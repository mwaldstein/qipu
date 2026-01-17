//! Text processing utilities for tokenization and ranking

use crate::lib::index::types::Index;
use std::collections::HashMap;

/// Simple word-based tokenizer splitting on non-alphanumeric characters
pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
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
