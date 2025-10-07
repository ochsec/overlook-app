use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::time::Duration;

const SEARXNG_SEARCH_ENDPOINT: &str = "/search";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDecision {
    pub search: bool,
    #[serde(default)]
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearxngResponse {
    results: Vec<SearxngResult>,
}

#[derive(Debug, Deserialize)]
struct SearxngResult {
    title: String,
    url: String,
    content: Option<String>,
}

pub struct WebSearchTool {
    client: reqwest::Client,
    base_url: String,
    api_token: String,
}

impl WebSearchTool {
    pub fn new() -> Result<Self> {
        let base_url = env::var("SEARXNG_URL")
            .map_err(|_| anyhow!("SEARXNG_URL environment variable not set"))?;

        let api_token = env::var("SEARXNG_API_TOKEN")
            .map_err(|_| anyhow!("SEARXNG_API_TOKEN environment variable not set"))?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Ok(Self { client, base_url, api_token })
    }

    pub async fn search(&self, query: &str, num_results: Option<usize>) -> Result<Vec<SearchResult>> {
        let url = format!("{}{}", self.base_url, SEARXNG_SEARCH_ENDPOINT);

        let response = self
            .client
            .get(&url)
            .header("X-API-Token", &self.api_token)
            .query(&[
                ("q", query),
                ("format", "json"),
            ])
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send search request: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("SearXNG API error {}: {}", status, error_text));
        }

        let searxng_response: SearxngResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse search response: {}", e))?;

        let mut results: Vec<SearchResult> = searxng_response
            .results
            .into_iter()
            .map(|r| SearchResult {
                title: r.title,
                url: r.url,
                content: r.content.unwrap_or_default(),
            })
            .collect();

        if let Some(limit) = num_results {
            results.truncate(limit);
        }

        Ok(results)
    }
}

#[derive(Clone)]
pub struct SearchOrchestrator {
    base_url: Option<String>,
    api_token: Option<String>,
}

impl SearchOrchestrator {
    pub fn new() -> Self {
        let base_url = std::env::var("SEARXNG_URL").ok();
        let api_token = std::env::var("SEARXNG_API_TOKEN").ok();

        if base_url.is_none() || api_token.is_none() {
            eprintln!("Warning: Web search not configured. Set SEARXNG_URL and SEARXNG_API_TOKEN environment variables.");
        }

        Self { base_url, api_token }
    }

    fn create_search_tool(&self) -> Result<WebSearchTool> {
        let base_url = self.base_url.as_ref()
            .ok_or_else(|| anyhow!("SEARXNG_URL not configured"))?;
        let api_token = self.api_token.as_ref()
            .ok_or_else(|| anyhow!("SEARXNG_API_TOKEN not configured"))?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Ok(WebSearchTool {
            client,
            base_url: base_url.clone(),
            api_token: api_token.clone(),
        })
    }

    /// Creates a user message that asks the LLM if it needs to search
    pub fn create_search_decision_message(user_query: &str) -> String {
        format!(r#"You have access to web search. For the following question, decide if you need to search for current information.

Question: "{}"

Respond with ONLY a JSON object:
{{"search": true, "query": "optimized search query"}} - if you need current data
{{"search": false}} - if you can answer from your training

Examples:
- "inflation numbers this year" → {{"search": true, "query": "US inflation rates 2025 monthly"}}
- "capital of France" → {{"search": false}}

Your decision (JSON only):"#, user_query)
    }

    /// Parses the LLM's response to extract search decision
    pub fn parse_search_decision(llm_response: &str) -> Result<SearchDecision> {
        // Try to extract JSON from the response
        // LLMs sometimes add extra text before/after the JSON
        let trimmed = llm_response.trim();

        // Look for JSON object boundaries
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                let json_str = &trimmed[start..=end];
                match serde_json::from_str::<SearchDecision>(json_str) {
                    Ok(decision) => return Ok(decision),
                    Err(e) => {
                        eprintln!("Failed to parse search decision JSON: {}", e);
                        eprintln!("JSON string: {}", json_str);
                    }
                }
            }
        }

        // Fallback: if we can't parse, assume no search needed
        eprintln!("Could not extract search decision from: {}", trimmed);
        Ok(SearchDecision {
            search: false,
            query: None,
        })
    }

    /// Performs a search and formats results for LLM context
    pub async fn search_and_format(&self, query: &str) -> Result<String> {
        let search_tool = self.create_search_tool()?;

        let results = search_tool.search(query, Some(10)).await?;

        if results.is_empty() {
            return Ok(format!(
                "Web search found no results for: {}\n\nPlease answer based on your training data.",
                query
            ));
        }

        // Format results for LLM
        let mut formatted = String::from("# Web Search Results\n\n");
        formatted.push_str(&format!("Query: {}\n\n", query));

        for (i, result) in results.iter().enumerate() {
            formatted.push_str(&format!("## Result {}\n", i + 1));
            formatted.push_str(&format!("**Title:** {}\n", result.title));
            formatted.push_str(&format!("**URL:** {}\n", result.url));
            if !result.content.is_empty() {
                formatted.push_str(&format!("**Content:** {}\n", result.content));
            }
            formatted.push_str("\n");
        }

        formatted.push_str("\nPlease use the above search results to answer the user's question. Cite sources when appropriate.\n");

        Ok(formatted)
    }
}
