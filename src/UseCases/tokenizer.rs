use crate::IO::ports::SentenceTokenizer;

const STOPWORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    "from", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do",
    "does", "did", "will", "would", "could", "should", "may", "might", "shall", "that", "this",
    "these", "those", "it", "its", "as", "if", "not", "no", "nor", "so", "yet", "both",
    "either", "just", "than", "then", "such", "when", "which", "who", "whom", "while", "where",
    "how", "what",
];

#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultTokenizer;

impl SentenceTokenizer for DefaultTokenizer {
    fn split_sentences(&self, text: &str) -> Vec<String> {
        split_sentences(text)
    }

    fn tokenize(&self, sentence: &str) -> Vec<String> {
        tokenize(sentence)
    }
}

pub fn split_sentences(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let len = bytes.len();

    let mut sentences = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;

    while i < len {
        // Delimitador: salto de línea doble (soporta \n\n y \r\n\r\n)
        if bytes[i] == b'\n' {
            if i + 1 < len && bytes[i + 1] == b'\n' {
                push_sentence(text, start, i, &mut sentences);
                i += 2;
                start = i;
                continue;
            }
        } else if bytes[i] == b'\r'
            && i + 3 < len
            && bytes[i + 1] == b'\n'
            && bytes[i + 2] == b'\r'
            && bytes[i + 3] == b'\n'
        {
            push_sentence(text, start, i, &mut sentences);
            i += 4;
            start = i;
            continue;
        }

        // Delimitador heurístico: salto de línea simple después de un título.
        if bytes[i] == b'\n'
            && (i == 0 || bytes[i - 1] != b'\r')
            && (i + 1 >= len || bytes[i + 1] != b'\n')
            && is_title_like(&text[start..i])
        {
            push_sentence(text, start, i, &mut sentences);
            i += 1;
            start = i;
            continue;
        }

        if bytes[i] == b'\r'
            && i + 1 < len
            && bytes[i + 1] == b'\n'
            && !(i + 3 < len && bytes[i + 2] == b'\r' && bytes[i + 3] == b'\n')
            && is_title_like(&text[start..i])
        {
            push_sentence(text, start, i, &mut sentences);
            i += 2;
            start = i;
            continue;
        }

        // Delimitador: . ! ? seguidos de whitespace o fin de string
        let b = bytes[i];
        if b == b'.' || b == b'!' || b == b'?' {
            let boundary = match bytes.get(i + 1) {
                None => true,
                Some(next) => next.is_ascii_whitespace(),
            };

            if boundary {
                let end = i + 1; // incluir puntuación
                push_sentence(text, start, end, &mut sentences);
                i = end;
                start = i;
                continue;
            }
        }

        i += 1;
    }

    push_sentence(text, start, len, &mut sentences);
    sentences
}

fn is_title_like(s: &str) -> bool {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Si ya parece una oración, no la tratamos como título.
    if trimmed.contains('.') || trimmed.contains('!') || trimmed.contains('?') {
        return false;
    }

    // Títulos suelen ser cortos.
    if trimmed.len() > 80 {
        return false;
    }

    let mut word_count = 0usize;
    let mut alpha_words = 0usize;
    let mut uppercase_start = 0usize;

    for word in trimmed.split_whitespace() {
        word_count += 1;
        if word_count > 6 {
            return false;
        }

        let Some(first) = word.chars().find(|c| c.is_alphanumeric()) else {
            continue;
        };

        if first.is_alphabetic() {
            alpha_words += 1;
            if first.is_uppercase() {
                uppercase_start += 1;
            }
        }
    }

    if alpha_words == 0 {
        return false;
    }

    // Requiere que al menos la mitad de las palabras alfabéticas empiecen en mayúscula.
    uppercase_start * 2 >= alpha_words
}

fn push_sentence(text: &str, start: usize, end: usize, out: &mut Vec<String>) {
    let slice = &text[start..end];
    let trimmed = slice.trim();
    if trimmed.is_empty() {
        return;
    }
    let token_count = trimmed.split_whitespace().count();
    if token_count < 5 {
        return;
    }
    out.push(trimmed.to_string());
}

pub fn tokenize(sentence: &str) -> Vec<String> {
    let lower = sentence.to_lowercase();
    let mut cleaned = String::with_capacity(lower.len());
    for ch in lower.chars() {
        if ch.is_alphanumeric() || ch == '\'' {
            cleaned.push(ch);
        } else {
            cleaned.push(' ');
        }
    }

    cleaned
        .split_whitespace()
        .filter(|tok| !STOPWORDS.contains(tok))
        .map(|tok| tok.to_string())
        .collect()
}
