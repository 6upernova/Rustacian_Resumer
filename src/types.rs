use std::path::PathBuf;

/// Representa un documento leído del disco.
#[derive(Debug, Clone)]
pub struct Document {
    pub path: PathBuf,
    pub content: String,
}

/// Oración tokenizada lista para ser puntuada.
#[derive(Debug, Clone)]
pub struct ProcessedSentence {
    pub raw: String,
    pub tokens: Vec<String>,
    pub source: PathBuf,
}

/// Oración con su puntaje TF-IDF final.
#[derive(Debug, Clone)]
pub struct ScoredSentence {
    pub raw: String,
    pub score: f64,
    pub source: PathBuf,
}
