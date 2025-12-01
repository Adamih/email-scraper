use tauri::State;
use url::Url;

use crate::db;
use crate::models::{AppState, Domain, SearchQuery};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub tags: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DomainResult {
    pub id: String,
    pub domain: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub crawl_status: String,
}

#[tauri::command]
pub async fn execute_google_search(
    state: State<'_, AppState>,
    request: SearchRequest,
) -> Result<Vec<DomainResult>, String> {
    let pool = &state.db;

    // Create search query record
    let search_query = SearchQuery::new(request.query.clone(), request.tags.clone());
    db::insert_search_query(pool, &search_query)
        .await
        .map_err(|e| e.to_string())?;

    // Get API key from config
    let api_key = db::get_config(pool, "google_api_key")
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Google API key not configured")?;

    let engine_id = db::get_config(pool, "google_search_engine_id")
        .await
        .map_err(|e| e.to_string())?
        .ok_or("Google Search Engine ID not configured")?;

    // Get number of results from config (default: 10, max: 10 per API limits)
    let num_results: u32 = db::get_config(pool, "google_search_results")
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);

    // Execute Google search
    let search_results = perform_google_search(&api_key, &engine_id, &request.query, num_results)
        .await
        .map_err(|e| format!("Search failed: {}", e))?;

    let mut domain_results = Vec::new();

    // Store domains in database
    for result in search_results {
        if let Ok(parsed_url) = Url::parse(&result.link) {
            let domain = parsed_url.domain().unwrap_or("unknown").to_string();

            let mut domain_record =
                Domain::new(search_query.id.clone(), domain.clone(), result.link);
            domain_record.title = Some(result.title);
            domain_record.description = Some(result.snippet);

            if let Ok(_) = db::insert_domain(pool, &domain_record).await {
                domain_results.push(DomainResult {
                    id: domain_record.id,
                    domain: domain_record.domain,
                    title: domain_record.title,
                    description: domain_record.description,
                    crawl_status: domain_record.crawl_status,
                });
            }
        }
    }

    Ok(domain_results)
}

#[tauri::command]
pub async fn get_searches(state: State<'_, AppState>) -> Result<Vec<SearchQuery>, String> {
    let pool = &state.db;
    db::get_all_search_queries(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_domains_for_search(
    state: State<'_, AppState>,
    search_id: String,
) -> Result<Vec<DomainResult>, String> {
    let pool = &state.db;
    let domains = db::get_domains_by_search_query(pool, &search_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok(domains
        .iter()
        .map(|d| DomainResult {
            id: d.id.clone(),
            domain: d.domain.clone(),
            title: d.title.clone(),
            description: d.description.clone(),
            crawl_status: d.crawl_status.clone(),
        })
        .collect())
}

async fn perform_google_search(
    api_key: &str,
    engine_id: &str,
    query: &str,
    num_results: u32,
) -> Result<Vec<crate::models::GoogleSearchResult>, String> {
    // Google Custom Search API limits num parameter to max 10 per request
    // but we can make multiple requests with different start indices to get more results
    // Google CSE API limit: max 100 results per day on free tier
    let num_results = num_results.min(100).max(1); // Cap at 100 total results
    let num_per_request = 10u32; // Always request 10 per API call (max allowed)
    let num_requests = (num_results + num_per_request - 1) / num_per_request; // Ceiling division

    // Build all requests in parallel using JoinSet
    let mut join_set = tokio::task::JoinSet::new();

    for page in 0..num_requests {
        let start_index = page * num_per_request + 1; // Google uses 1-based indexing
        let api_key = api_key.to_string();
        let engine_id = engine_id.to_string();
        let query = query.to_string();

        join_set.spawn(async move {
            let client = reqwest::Client::new();
            let url = format!(
                "https://www.googleapis.com/customsearch/v1?q={}&key={}&cx={}&num={}&start={}",
                urlencoding::encode(&query),
                api_key,
                engine_id,
                num_per_request,
                start_index
            );

            let response = client
                .get(&url)
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                return Err(format!(
                    "API returned status: {} - {} (request {} of {})",
                    status,
                    body,
                    page + 1,
                    num_requests
                ));
            }

            let json: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            let mut page_results = Vec::new();
            if let Some(items) = json.get("items").and_then(|v| v.as_array()) {
                for item in items {
                    if let (Some(title), Some(link), Some(snippet)) = (
                        item.get("title").and_then(|v| v.as_str()),
                        item.get("link").and_then(|v| v.as_str()),
                        item.get("snippet").and_then(|v| v.as_str()),
                    ) {
                        page_results.push(crate::models::GoogleSearchResult {
                            title: title.to_string(),
                            link: link.to_string(),
                            snippet: snippet.to_string(),
                        });
                    }
                }
            }

            Ok(page_results)
        });
    }

    let mut all_results = Vec::new();

    // Collect results as they complete (they may come back in any order)
    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(page_results)) => {
                for item in page_results {
                    all_results.push(item);
                    // Stop if we've reached the desired number of results
                    if all_results.len() >= num_results as usize {
                        return Ok(all_results);
                    }
                }
            }
            Ok(Err(e)) => {
                // Log the error but continue with partial results
                log::warn!("Error fetching search page: {}", e);
            }
            Err(e) => {
                log::warn!("Task join error: {}", e);
            }
        }
    }

    Ok(all_results)
}

#[tauri::command]
pub async fn delete_search(
    state: State<'_, AppState>,
    search_id: String,
) -> Result<String, String> {
    let pool = &state.db;

    // Delete all emails associated with this search
    db::delete_emails_by_search(pool, &search_id)
        .await
        .map_err(|e| e.to_string())?;

    // Delete all domains associated with this search
    db::delete_domains_by_search(pool, &search_id)
        .await
        .map_err(|e| e.to_string())?;

    // Delete the search query itself
    db::delete_search_query(pool, &search_id)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Search deleted successfully".to_string())
}
