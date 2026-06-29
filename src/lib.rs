pub mod types;

pub mod IO {
    pub mod input;
    pub mod output;
    pub mod ports;
    pub mod external;
}

pub mod UseCases {
    pub mod processor;
    pub mod summarizer;
    pub mod tokenizer;
    pub mod entities;
    pub mod additionalInfo;
}

pub use types::{Document, ProcessedSentence, ScoredSentence};
