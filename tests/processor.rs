use resumidor_rust::processor::{compute_idf, score_sentence, select_top_sentences};
use resumidor_rust::ProcessedSentence;

use std::collections::HashMap;
use std::path::PathBuf;

fn ps(raw: &str, tokens: &[&str]) -> ProcessedSentence {
    ProcessedSentence {
        raw: raw.to_string(),
        tokens: tokens.iter().map(|t| t.to_string()).collect(),
        source: PathBuf::from("doc.txt"),
    }
}

#[test]
fn test_idf_single_sentence() {
    let sentences = vec![ps("Hello", &["hello"])];
    let idf = compute_idf(&sentences);
    let got = idf.get("hello").copied().unwrap();
    let expected = (1.0_f64 / 2.0_f64).ln();
    assert!((got - expected).abs() < 1e-9);
}

#[test]
fn test_score_empty_tokens() {
    let sentence = ps("Empty", &[]);
    let idf: HashMap<String, f64> = HashMap::new();
    let got = score_sentence(&sentence, &idf);
    assert_eq!(got, 0.0);
}

#[test]
fn test_select_top_n_returns_correct_count() {
    let sentences = vec![ps("A", &["apple"]), ps("B", &["banana"]), ps("C", &["apple"])];
    let mut idf = HashMap::new();
    idf.insert("apple".to_string(), 1.0);
    idf.insert("banana".to_string(), 1.0);

    let top = select_top_sentences(sentences, &idf, 2);
    assert_eq!(top.len(), 2);
}

#[test]
fn test_select_top_n_ordered_descending() {
    let sentences = vec![
        ps("S1", &["apple"]),
        ps("S2", &["banana"]),
        ps("S3", &["apple", "banana"]),
    ];
    let mut idf = HashMap::new();
    idf.insert("apple".to_string(), 1.0);
    idf.insert("banana".to_string(), 0.5);

    let top = select_top_sentences(sentences, &idf, 3);
    assert_eq!(top[0].raw, "S1");
    assert_eq!(top[1].raw, "S3");
    assert_eq!(top[2].raw, "S2");
    assert!(top[0].score >= top[1].score);
    assert!(top[1].score >= top[2].score);
}

#[test]
fn test_select_top_n_tie_preserves_order() {
    let sentences = vec![ps("A", &["x"]), ps("B", &["x"])];
    let mut idf = HashMap::new();
    idf.insert("x".to_string(), 1.0);

    let top = select_top_sentences(sentences, &idf, 2);
    assert_eq!(top[0].raw, "A");
    assert_eq!(top[1].raw, "B");
}
