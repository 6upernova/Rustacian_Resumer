use crate::IO::external::WikipediaApiClient;
use crate::UseCases::entities::NerResult;
use std::collections::HashMap;
use urlencoding::encode;

/// Estructura que mapea cada entidad a su descripción de Wikipedia.
pub type EntityInfo = HashMap<String, String>;

/// Texto por defecto cuando Wikipedia no devuelve información.
const DEFAULT_INFO: &str = "No se encontro informacion";

/// Obtiene información adicional de Wikipedia para todas las entidades detectadas.
///
/// Recorre todos los documentos y sus entidades, consulta la API de Wikipedia
/// y construye un mapa `entidad -> descripción`.
pub async fn fetch_entity_info(ner_result: &NerResult) -> EntityInfo {
    let client = WikipediaApiClient::new();
    let mut entity_info = EntityInfo::new();

    for (_doc_name, doc_entities) in ner_result {
        for entity in &doc_entities.entities {
            // Evitar consultas duplicadas para la misma entidad
            if entity_info.contains_key(&entity.text) {
                continue;
            }
            let encoded_url = encode(&entity.text);
            let description = match client.get(&encoded_url).await {
                Ok(Some(summary)) => summary.extract,
                Ok(None) | Err(_) => DEFAULT_INFO.to_string(),
            };

            entity_info.insert(entity.text.clone(), description);
        }
    }

    entity_info
}
