use std::env;
use std::path::PathBuf;

use resumidor_rust::input::StdFileSystem;
use resumidor_rust::output;
use resumidor_rust::processor::TfidfRanker;
use resumidor_rust::summarizer::Summarizer;
use resumidor_rust::tokenizer::DefaultTokenizer;

fn main() {
    let dir_arg = env::args().nth(1).unwrap_or_else(|| "docs".to_string());
    let dir = PathBuf::from(dir_arg);

    let summarizer = Summarizer::new(
        StdFileSystem,
        DefaultTokenizer,
        TfidfRanker,
    );

    let report = match summarizer.summarize_dir(&dir, 10) {
        Ok(report) => report,
        Err(e) => output::fatal_error(&e.to_string()),
    };

    for w in &report.warnings {
        output::print_warning(w);
    }

    output::print_header(&dir, report.file_count);
    output::print_summary(&report.top_sentences);
}
