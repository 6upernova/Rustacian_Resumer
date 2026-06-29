use std::path::PathBuf;
use serde::{Deserialize, Serialize};

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

/// Estructura del retorno de la API de Wikipedia
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SummaryJSONResponse {
    #[serde(rename = "type")]
    pub page_type: String,
    pub title: String,
    pub displaytitle: String,
    pub namespace: Namespace,
    pub wikibase_item: String,
    pub titles: Titles,
    pub pageid: u64,
    #[serde(default)]
    pub thumbnail: Option<Thumbnail>,
    #[serde(default)]
    pub originalimage: Option<OriginalImage>,
    pub lang: String,
    pub dir: String,
    pub revision: String,
    pub tid: String,
    pub timestamp: String, // ISO 8601, could use DateTime<Utc> with serde_with
    pub description: String,
    pub description_source: String,
    pub content_urls: ContentUrls,
    pub extract: String,
    pub extract_html: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Namespace {
    pub id: i32,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Titles {
    pub canonical: String,
    pub normalized: String,
    pub display: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Thumbnail {
    pub source: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OriginalImage {
    pub source: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContentUrls {
    pub desktop: UrlSet,
    pub mobile: UrlSet,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UrlSet {
    pub page: String,
    pub revisions: String,
    pub edit: String,
    pub talk: String,
}

//Para deserializar el json de ner_detector

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub text: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentEntities {
    pub entities: Vec<Entity>,
}
