use std::collections::HashMap;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

use crate::io::ports::{FileSystem, SentenceRanker, SentenceTokenizer};
use crate::{ProcessedSentence, ScoredSentence};

#[derive(Debug)]
pub enum SummarizeError {
    ListFailed { dir: PathBuf, source: io::Error },
    NoTxtFiles { dir: PathBuf },
    NoValidSentences,
}

impl fmt::Display for SummarizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SummarizeError::ListFailed { dir, source } => write!(
                f,
                "No se pudo listar archivos .txt en {}: {}",
                dir.display(),
                source
            ),
            SummarizeError::NoTxtFiles { dir } => write!(
                f,
                "No se encontraron archivos .txt en el directorio {}",
                dir.display()
            ),
            SummarizeError::NoValidSentences => {
                write!(f, "No se encontraron oraciones válidas para resumir.")
            }
        }
    }
}

impl std::error::Error for SummarizeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SummarizeError::ListFailed { source, .. } => Some(source),
            SummarizeError::NoTxtFiles { .. } | SummarizeError::NoValidSentences => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SummaryReport {
    pub file_count: usize,
    pub warnings: Vec<String>,
    pub top_sentences: Vec<ScoredSentence>,
}

enum WorkerMessage {
    Sentences(Vec<ProcessedSentence>),
    Warning(String),
}

#[derive(Debug, Clone)]
pub struct Summarizer<I, T, R> {
    fs: I,
    tokenizer: T,
    ranker: R,
}

impl<I, T, R> Summarizer<I, T, R> {
    pub fn new(fs: I, tokenizer: T, ranker: R) -> Self {
        Self { fs, tokenizer, ranker }
    }
}

impl<I, T, R> Summarizer<I, T, R>
where
    I: FileSystem + Clone + Send + Sync + 'static,
    T: SentenceTokenizer + Clone + Send + Sync + 'static,
    R: SentenceRanker + Clone + Send + Sync + 'static,
{
    pub fn summarize_dir(&self, dir: &Path, top_n: usize) -> Result<SummaryReport, SummarizeError> {
        let paths = self
            .fs
            .list_txt_files(dir)
            .map_err(|e| SummarizeError::ListFailed {
                dir: dir.to_path_buf(),
                source: e,
            })?;

        if paths.is_empty() {
            return Err(SummarizeError::NoTxtFiles {
                dir: dir.to_path_buf(),
            });
        }

        let file_count = paths.len();
        let (tx, rx) = mpsc::channel::<WorkerMessage>();
        let mut handles = Vec::with_capacity(file_count);

        for path in paths {
            let tx = tx.clone();
            let fs = self.fs.clone();
            let tokenizer = self.tokenizer.clone();

            let handle = thread::spawn(move || {
                let message = match fs.read_file(&path) {
                    Ok(content) => {
                        let sentences_raw = tokenizer.split_sentences(&content);
                        let processed = sentences_raw
                            .into_iter()
                            .map(|raw| {
                                let tokens = tokenizer.tokenize(&raw);
                                ProcessedSentence {
                                    raw,
                                    tokens,
                                    source: path.clone(),
                                }
                            })
                            .filter(|ps| !ps.tokens.is_empty())
                            .collect::<Vec<_>>();

                        WorkerMessage::Sentences(processed)
                    }
                    Err(e) => WorkerMessage::Warning(format!("No se pudo leer {:?}: {}", path, e)),
                };

                let _ = tx.send(message);
            });

            handles.push(handle);
        }

        drop(tx);

        for h in handles {
            h.join().expect("El hilo de procesamiento falló");
        }

        let mut warnings = Vec::new();
        let mut all_sentences = Vec::new();
        for msg in rx {
            match msg {
                WorkerMessage::Sentences(mut sentences) => all_sentences.append(&mut sentences),
                WorkerMessage::Warning(w) => warnings.push(w),
            }
        }

        if all_sentences.is_empty() {
            return Err(SummarizeError::NoValidSentences);
        }

        let idf: HashMap<String, f64> = self.ranker.compute_idf(&all_sentences);
        let top_sentences = self
            .ranker
            .select_top_sentences(all_sentences, &idf, top_n);

        Ok(SummaryReport {
            file_count,
            warnings,
            top_sentences,
        })
    }
}
