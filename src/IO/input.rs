use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::IO::ports::FileSystem;

pub fn list_txt_files(dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_file() {
            continue;
        }

        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if ext.eq_ignore_ascii_case("txt") {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

pub fn read_file(path: &Path) -> Result<String, io::Error> {
    fs::read_to_string(path)
}

/// Lista las subcarpetas inmediatas dentro de `root` que pueden usarse
/// como temáticas (cada carpeta debe contener archivos .txt).
///
/// Se ignoran entradas que no son directorios y se ocultas (`.` / `..`).
pub fn list_topics(root: &Path) -> Result<Vec<PathBuf>, io::Error> {
    let mut topics = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if !file_type.is_dir() {
            continue;
        }

        let path = entry.path();
        let name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if name.starts_with('.') {
            continue;
        }

        topics.push(path);
    }

    topics.sort_by(|a, b| {
        let an = a.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let bn = b.file_name().and_then(|n| n.to_str()).unwrap_or("");
        an.cmp(bn)
    });

    Ok(topics)
}

/// Selecciona una carpeta de temática dentro de `docs_root`.
///
/// - Si el usuario pasa un argumento no vacío, se usa esa carpeta
///   (`docs_root/<arg>`).
/// - Si no, se listan las temáticas disponibles y se le pide al usuario
///   elegir por stdin (1..N). Si hay una sola, se elige automáticamente.
/// - Si no hay ninguna carpeta válida, devuelve un error.
pub fn select_topic(docs_root: &Path, requested: Option<&str>) -> Result<PathBuf, io::Error> {
    if let Some(name) = requested {
        let name = name.trim();
        if !name.is_empty() {
            let candidate = docs_root.join(name);
            if !candidate.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "La temática '{}' no existe dentro de {}",
                        name,
                        docs_root.display()
                    ),
                ));
            }
            return Ok(candidate);
        }
    }

    let topics = list_topics(docs_root)?;
    if topics.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "No se encontraron carpetas de temáticas dentro de {}",
                docs_root.display()
            ),
        ));
    }

    if topics.len() == 1 {
        return Ok(topics.into_iter().next().unwrap());
    }

    println!("Tematicas disponibles:");
    for (i, topic) in topics.iter().enumerate() {
        let name = topic
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("(sin nombre)");
        println!("  [{}] {}", i + 1, name);
    }
    print!("Seleccione una tematica (1-{}): ", topics.len());
    use std::io::Write;
    let _ = std::io::stdout().flush();

    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer)?;
    let choice: usize = buffer
        .trim()
        .parse()
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Entrada invalida: '{}'", buffer.trim()),
            )
        })?;

    if choice == 0 || choice > topics.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Opcion fuera de rango: {} (esperado 1..={})",
                choice,
                topics.len()
            ),
        ));
    }

    Ok(topics.into_iter().nth(choice - 1).unwrap())
}

#[derive(Debug, Clone, Copy, Default)]
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn list_txt_files(&self, dir: &Path) -> Result<Vec<PathBuf>, io::Error> {
        list_txt_files(dir)
    }

    fn read_file(&self, path: &Path) -> Result<String, io::Error> {
        read_file(path)
    }
}
