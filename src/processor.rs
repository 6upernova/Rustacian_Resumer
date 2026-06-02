use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::ports::SentenceRanker;
use crate::{ProcessedSentence, ScoredSentence};

#[derive(Debug, Clone, Copy, Default)]
pub struct TfidfRanker;

impl SentenceRanker for TfidfRanker {
    fn compute_idf(&self, sentences: &[ProcessedSentence]) -> HashMap<String, f64> {
        compute_idf(sentences)
    }

    fn select_top_sentences(
        &self,
        sentences: Vec<ProcessedSentence>,
        idf: &HashMap<String, f64>,
        top_n: usize,
    ) -> Vec<ScoredSentence> {
        select_top_sentences(sentences, idf, top_n)
    }
}

pub fn compute_idf(sentences: &[ProcessedSentence]) -> HashMap<String, f64> {
    let n = sentences.len();
    if n == 0 {
        return HashMap::new();
    }

    let mut df: HashMap<String, usize> = HashMap::new();
    for sentence in sentences {
        let mut seen: HashSet<&str> = HashSet::new();
        for token in &sentence.tokens {
            if seen.insert(token.as_str()) {
                *df.entry(token.clone()).or_insert(0) += 1;
            }
        }
    }

    let n_f = n as f64;
    df.into_iter()
        .map(|(token, df_count)| {
            let idf = (n_f / (1.0 + df_count as f64)).ln();
            (token, idf)
        })
        .collect()
}

pub fn score_sentence(sentence: &ProcessedSentence, idf: &HashMap<String, f64>) -> f64 {
    if sentence.tokens.is_empty() {
        return 0.0;
    }

    let total = sentence.tokens.len() as f64;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for token in &sentence.tokens {
        *counts.entry(token.as_str()).or_insert(0) += 1;
    }

    let mut score = 0.0;
    for (token, count) in counts {
        let tf = count as f64 / total;
        let idf_val = idf.get(token).copied().unwrap_or(0.0);
        score += tf * idf_val;
    }
    score
}

pub fn select_top_sentences(
    sentences: Vec<ProcessedSentence>,
    idf: &HashMap<String, f64>,
    top_n: usize,
) -> Vec<ScoredSentence> {
    let mut scored: Vec<(usize, ScoredSentence)> = sentences
        .into_iter()
        .enumerate()
        .map(|(idx, sentence)| {
            let score = score_sentence(&sentence, idf);
            let scored = ScoredSentence {
                raw: sentence.raw,
                score,
                source: sentence.source,
            };
            (idx, scored)
        })
        .collect();

    scored.sort_by(|(idx_a, a), (idx_b, b)| {
        let score_cmp = b.score.total_cmp(&a.score);
        if score_cmp == Ordering::Equal {
            idx_a.cmp(idx_b)
        } else {
            score_cmp
        }
    });

    scored
        .into_iter()
        .take(top_n)
        .map(|(_, s)| s)
        .collect()
}
