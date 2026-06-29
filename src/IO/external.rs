use crate::types::SummaryJSONResponse;

pub struct WikipediaApiClient {
    client: reqwest::Client,
    base_url: String,
}

impl WikipediaApiClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://es.wikipedia.org/api/rest_v1/page/summary".to_string(),
        }
    }

    /// Busca el resumen de una entidad en Wikipedia.
    /// Retorna `Ok(Some(response))` si se encuentra, `Ok(None)` si no existe (404),
    /// o un error si falla la request.
    pub async fn get(&self, entity: &str) -> Result<Option<SummaryJSONResponse>, reqwest::Error> {
        let url = format!("{}/{}", self.base_url, entity);
        let response = self.client.get(&url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        let summary: SummaryJSONResponse = response.json().await?;
        Ok(Some(summary))
    }
}