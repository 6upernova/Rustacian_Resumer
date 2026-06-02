use resumidor_rust::tokenizer::{split_sentences, tokenize};

#[test]
fn test_tokenize_removes_stopwords() {
    let got = tokenize("This is a test of the tokenizer.");
    assert_eq!(got, vec!["test".to_string(), "tokenizer".to_string()]);
}

#[test]
fn test_tokenize_lowercases() {
    let got = tokenize("Hello WORLD");
    assert_eq!(got, vec!["hello".to_string(), "world".to_string()]);
}

#[test]
fn test_split_sentences_basic() {
    let input =
        "One two three four five. Six seven eight nine ten!\n\nEleven twelve thirteen fourteen fifteen?";
    let got = split_sentences(input);
    assert_eq!(
        got,
        vec![
            "One two three four five.".to_string(),
            "Six seven eight nine ten!".to_string(),
            "Eleven twelve thirteen fourteen fifteen?".to_string(),
        ]
    );
}

#[test]
fn test_split_sentences_empty_input() {
    let got = split_sentences("");
    assert!(got.is_empty());
}

#[test]
fn test_split_sentences_splits_title_newline() {
    let input = "Instrumentos de Fomento Nacional\nA nivel local, los gobiernos implementan diversas herramientas legales y regulatorias para incentivar la inversión.";
    let got = split_sentences(input);
    assert_eq!(
        got,
        vec![
            "A nivel local, los gobiernos implementan diversas herramientas legales y regulatorias para incentivar la inversión.".to_string(),
        ]
    );
}
