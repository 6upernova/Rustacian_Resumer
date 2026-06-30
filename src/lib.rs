pub mod types;

pub mod io {
    pub mod input;
    pub mod output;
    pub mod ports;
    pub mod external;
}

pub mod use_cases {
    pub mod processor;
    pub mod summarizer;
    pub mod tokenizer;
    pub mod entities;
    pub mod additional_info;
}

pub use types::{Document, ProcessedSentence, ScoredSentence};
