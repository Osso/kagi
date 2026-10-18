use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://kagi.com/api/v0";

pub struct KagiClient {
    http: reqwest::Client,
    token: String,
}

// --- FastGPT ---

#[derive(Serialize)]
struct FastGptRequest<'a> {
    query: &'a str,
    cache: bool,
    web_search: bool,
}

#[derive(Deserialize)]
pub struct FastGptResponse {
    pub data: FastGptData,
}

#[derive(Deserialize)]
pub struct FastGptData {
    pub output: String,
    pub tokens: u64,
    pub references: Vec<Reference>,
}

#[derive(Deserialize)]
pub struct Reference {
    pub title: String,
    pub snippet: String,
    pub url: String,
}

// --- Summarizer ---

#[derive(Serialize)]
struct SummarizeRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    engine: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_language: Option<&'a str>,
}

#[derive(Deserialize)]
pub struct SummarizeResponse {
    pub data: SummarizeData,
}

#[derive(Deserialize)]
pub struct SummarizeData {
    pub output: String,
    pub tokens: u64,
}

// --- Web Search ---

#[derive(Deserialize)]
pub struct SearchResponse {
    pub data: Vec<SearchResult>,
}

#[derive(Deserialize)]
pub struct SearchResult {
    pub url: String,
    pub title: String,
    pub snippet: Option<String>,
    pub published: Option<String>,
}

// --- Client ---

impl KagiClient {
    pub fn new(token: String) -> Self {
        let http = reqwest::Client::new();
        Self { http, token }
    }

    fn auth_header(&self) -> String {
        format!("Bot {}", self.token)
    }

    pub async fn fastgpt(&self, query: &str, cache: bool, web_search: bool) -> Result<FastGptResponse> {
        let body = FastGptRequest { query, cache, web_search };
        let resp = self
            .http
            .post(format!("{BASE_URL}/fastgpt"))
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await
            .context("Failed to send FastGPT request")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("FastGPT API error {status}: {text}");
        }

        resp.json::<FastGptResponse>().await.context("Failed to parse FastGPT response")
    }

    pub async fn summarize(
        &self,
        url: Option<&str>,
        text: Option<&str>,
        summary_type: Option<&str>,
        target_language: Option<&str>,
    ) -> Result<SummarizeResponse> {
        let body = SummarizeRequest {
            url,
            text,
            engine: None,
            summary_type,
            target_language,
        };
        let resp = self
            .http
            .post(format!("{BASE_URL}/summarize"))
            .header("Authorization", self.auth_header())
            .json(&body)
            .send()
            .await
            .context("Failed to send Summarize request")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Summarize API error {status}: {text}");
        }

        resp.json::<SummarizeResponse>().await.context("Failed to parse Summarize response")
    }

    pub async fn search(&self, query: &str, limit: Option<u32>) -> Result<SearchResponse> {
        let mut req = self
            .http
            .get(format!("{BASE_URL}/search"))
            .header("Authorization", self.auth_header())
            .query(&[("q", query)]);

        if let Some(l) = limit {
            req = req.query(&[("limit", l.to_string())]);
        }

        let resp = req.send().await.context("Failed to send Search request")?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("Search API error {status}: {text}");
        }

        resp.json::<SearchResponse>().await.context("Failed to parse Search response")
    }
}
