use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

use crate::{ProcessedSentence, ScoredSentence};

/// Abstracción de acceso al sistema de archivos.
///
/// Permite testear/invertir dependencias sin acoplar la lógica a `std::fs`.
pub trait FileSystem: Send + Sync {
    fn list_txt_files(&self, dir: &Path) -> Result<Vec<PathBuf>, io::Error>;
    fn read_file(&self, path: &Path) -> Result<String, io::Error>;
}

/// Abstracción de segmentación y tokenización.
pub trait SentenceTokenizer: Send + Sync {
    fn split_sentences(&self, text: &str) -> Vec<String>;
    fn tokenize(&self, sentence: &str) -> Vec<String>;
}

/// Abstracción de ranking/selección de oraciones.
pub trait SentenceRanker: Send + Sync {
    fn compute_idf(&self, sentences: &[ProcessedSentence]) -> HashMap<String, f64>;

    fn select_top_sentences(
        &self,
        sentences: Vec<ProcessedSentence>,
        idf: &HashMap<String, f64>,
        top_n: usize,
    ) -> Vec<ScoredSentence>;
}
