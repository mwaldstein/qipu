use crate::lib::index::types::Index;
use std::collections::HashMap;

pub fn cosine_similarity(vec_a: &HashMap<String, f64>, vec_b: &HashMap<String, f64>) -> f64 {
    let mut dot_product = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (term, weight) in vec_a {
        norm_a += weight * weight;
        if let Some(weight_b) = vec_b.get(term) {
            dot_product += weight * weight_b;
        }
    }

    for weight in vec_b.values() {
        norm_b += weight * weight;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot_product / (norm_a.sqrt() * norm_b.sqrt())
}

pub fn get_tfidf_vector(index: &Index, term_freqs: &HashMap<String, f64>) -> HashMap<String, f64> {
    let mut vector = HashMap::new();
    let total_docs = index.total_docs as f64;

    for (term, &tf) in term_freqs {
        let df = *index.term_df.get(term).unwrap_or(&1) as f64;
        let idf = ((total_docs + 1.0) / (df + 1.0)).ln() + 1.0;

        let weight = tf * idf;
        vector.insert(term.clone(), weight);
    }

    vector
}
