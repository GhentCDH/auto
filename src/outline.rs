/*!
 * Minimal Outline wiki HTTP client for document sync.
 */

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{AppState, Error, Result, overview::Overview as _};

pub struct OutlineClient {
    base_url: Url,
    api_key: String,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct OutlineResponse<T> {
    data: T,
}

#[derive(Debug, Deserialize)]
pub struct OutlineDocument {
    pub id: String,
    pub title: String,
    pub text: String,
}

#[derive(Serialize)]
struct DocInfoRequest<'a> {
    id: &'a str,
}

#[derive(Serialize)]
struct DocUpdateRequest<'a> {
    id: &'a str,
    text: &'a str,
}

impl OutlineClient {
    pub fn new(base_url: &Url, api_key: &str) -> Self {
        Self {
            base_url: base_url.clone(),
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// Extract the document ID from an Outline URL.
    /// Outline URLs look like: `https://docs.example.com/doc/some-title-fi9aj1Foeq`
    /// The last path segment's final hyphen-separated part is the `urlId`.
    pub fn extract_doc_id(url: &str) -> Result<String> {
        let parsed = Url::parse(url)
            .map_err(|_| Error::ValidationError("Invalid Outline URL".to_string()))?;

        let path = parsed.path().trim_end_matches('/');
        let last_segment = path
            .rsplit('/')
            .next()
            .ok_or_else(|| Error::ValidationError("No path in Outline URL".to_string()))?;

        // The urlId is the last hyphen-separated token in the slug
        let doc_id = last_segment.rsplit('-').next().unwrap_or(last_segment);

        if doc_id.is_empty() {
            return Err(Error::ValidationError(
                "Empty document ID in URL".to_string(),
            ));
        }

        Ok(doc_id.to_string())
    }

    pub async fn get_document(&self, doc_id: &str) -> Result<OutlineDocument> {
        let url = self
            .base_url
            .join("/api/documents.info")
            .map_err(|e| Error::InternalError(format!("URL join error: {e}")))?;

        let resp = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&DocInfoRequest { id: doc_id })
            .send()
            .await
            .map_err(|e| Error::InternalError(format!("Outline request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::InternalError(format!(
                "Outline API error {status}: {body}"
            )));
        }

        let wrapper: OutlineResponse<OutlineDocument> = resp
            .json()
            .await
            .map_err(|e| Error::InternalError(format!("Outline response parse error: {e}")))?;

        Ok(wrapper.data)
    }

    pub async fn update_document(&self, doc_id: &str, text: &str) -> Result<()> {
        let url = self
            .base_url
            .join("/api/documents.update")
            .map_err(|e| Error::InternalError(format!("URL join error: {e}")))?;

        let resp = self
            .client
            .post(url)
            .bearer_auth(&self.api_key)
            .json(&DocUpdateRequest { id: doc_id, text })
            .send()
            .await
            .map_err(|e| Error::InternalError(format!("Outline request failed: {e}")))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(Error::InternalError(format!(
                "Outline API error {status}: {body}"
            )));
        }

        Ok(())
    }
}

pub async fn sync_application(state: &AppState, id: &str) -> Result<()> {
    let (Some(outline_url), Some(outline_api_key)) =
        (&state.config.outline_url, &state.config.outline_api_key)
    else {
        return Err(crate::Error::ValidationError(
            "Outline not configured (set OUTLINE_URL and OUTLINE_API_KEY)".to_string(),
        ));
    };

    let app = crate::service::application::get_with_relations(&state.pool, id).await?;

    let entity_outline_url = app.application.outline_url.as_deref().ok_or_else(|| {
        crate::Error::ValidationError("No Outline document linked to this application".to_string())
    })?;

    let doc_id = crate::outline::OutlineClient::extract_doc_id(entity_outline_url)?;
    let client = crate::outline::OutlineClient::new(outline_url, outline_api_key);

    let doc = client.get_document(&doc_id).await?;
    let new_text = app.splice_overview(state, &doc.text);
    client.update_document(&doc_id, &new_text).await?;

    Ok(())
}

pub async fn sync_service(state: &AppState, id: &str) -> Result<()> {
    let (Some(outline_url), Some(outline_api_key)) =
        (&state.config.outline_url, &state.config.outline_api_key)
    else {
        return Err(crate::Error::ValidationError(
            "Outline not configured (set OUTLINE_URL and OUTLINE_API_KEY)".to_string(),
        ));
    };

    let svc = crate::service::service::get_with_relations(&state.pool, id).await?;

    let entity_outline_url = svc.service.outline_url.as_deref().ok_or_else(|| {
        crate::Error::ValidationError("No Outline document linked to this service".to_string())
    })?;

    let doc_id = crate::outline::OutlineClient::extract_doc_id(entity_outline_url)?;
    let client = crate::outline::OutlineClient::new(outline_url, outline_api_key);

    let doc = client.get_document(&doc_id).await?;
    let new_text = svc.splice_overview(state, &doc.text);
    client.update_document(&doc_id, &new_text).await?;

    Ok(())
}
