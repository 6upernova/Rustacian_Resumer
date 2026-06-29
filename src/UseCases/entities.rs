
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use std::{collections::HashMap, process::Stdio};
use std::io;
use std::path::Path;

use serde_json::Value;

use crate::IO::input::{list_txt_files, read_file};
use crate::types::DocumentEntities;

pub type NerResult = HashMap<String, DocumentEntities>;

fn topic_to_json(topic_dir: &Path) -> Result<Value, io::Error> {
    let mut docs = HashMap::new();

    let files = list_txt_files(topic_dir)?;

    for file in files {
        let content = read_file(&file)?;

        let key = file
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        
        docs.insert(key, content);
    }

    Ok(serde_json::to_value(docs).unwrap())
}


async fn execute_ner(
        input_json: &str,
        script_path: &str,
) -> Result<NerResult, Box<dyn std::error::Error + Send + Sync>> {


    let mut child = Command::new("python")
        .arg(script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or("no se pudo abrir stdin")?;

    stdin.write_all(input_json.as_bytes()).await?;
    drop(stdin);

    let output=child.wait_with_output().await?;

    if !output.status.success(){
        return Err(format!(
            "El script termino con codigo {:?}",
            output.status.code()
        ).into());
    }

    Ok(serde_json::from_slice::<NerResult>(&output.stdout)?)
}


pub async fn search_entities(topic_dir: &Path, script_path: &str) -> Result<NerResult, Box<dyn std::error::Error + Send + Sync>> {
    let json_value = topic_to_json(topic_dir)?;
    let input_json = serde_json::to_string(&json_value)?;
    execute_ner(&input_json, script_path).await
}