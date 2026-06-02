use std::path::Path;

use crate::ScoredSentence;

pub fn print_header(dir: &Path, file_count: usize) {
    println!("============================================================");
    println!("  Resumidor TF-IDF");
    println!(
        "  Directorio: {}  |  Archivos procesados: {}",
        dir.display(),
        file_count
    );
    println!("============================================================");
    println!();
}

pub fn print_summary(sentences: &[ScoredSentence]) {
    println!(
        "=== RESUMEN EXTRACTIVO (top {} oraciones) ===",
        sentences.len()
    );
    println!();

    for (i, s) in sentences.iter().enumerate() {
        let file_name = s
            .source
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("(desconocido)");

        println!(
            "[{}] score: {:.4}  |  fuente: {}",
            i + 1,
            s.score,
            file_name
        );
        println!("    {}", s.raw);
        println!();
    }
}

pub fn print_warning(msg: &str) {
    eprintln!("[WARN] {msg}");
}

pub fn fatal_error(msg: &str) -> ! {
    eprintln!("[ERROR] {msg}");
    std::process::exit(1)
}
