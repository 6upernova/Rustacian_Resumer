use std::env;
use std::path::PathBuf;

use resumidor_rust::IO::input::{self, StdFileSystem};
use resumidor_rust::IO::output;
use resumidor_rust::UseCases::additionalInfo;
use resumidor_rust::UseCases::entities;
use resumidor_rust::UseCases::processor::TfidfRanker;
use resumidor_rust::UseCases::summarizer::Summarizer;
use resumidor_rust::UseCases::tokenizer::DefaultTokenizer;

#[tokio::main]
async fn main() {
    // Obtener la raíz de docs (por defecto ./docs)
    let docs_root = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("docs"));

    let script_path = "./ner_detector_v2.py";
    // Obtener la temática opcional (segundo argumento)
    let topic_arg = env::args().nth(2);

    // Seleccionar la carpeta de temática
    let topic_dir = match input::select_topic(&docs_root, topic_arg.as_deref()) {
        Ok(dir) => dir,
        Err(e) => output::fatal_error(&e.to_string()),
    };

    let summarizer = Summarizer::new(
        StdFileSystem,
        DefaultTokenizer,
        TfidfRanker,
    );

    let entities_report  = match entities::search_entities(&topic_dir, script_path).await {
        Ok(entities_report) => entities_report,
        Err(e) => output::fatal_error(&e.to_string()),
    };

    // Extraer información adicional de Wikipedia para las entidades encontradas
    let entity_info = additionalInfo::fetch_entity_info(&entities_report).await;

    let report = match summarizer.summarize_dir(&topic_dir, 10) {
        Ok(report) => report,
        Err(e) => output::fatal_error(&e.to_string()),
    };

    for w in &report.warnings {
        output::print_warning(w);
    }

    output::print_header(&topic_dir, report.file_count);
    output::print_summary(&report.top_sentences);
    
    // Imprimir información de entidades si hay alguna
    if !entity_info.is_empty() {
        output::print_entity_info(&entity_info);
    }
}
