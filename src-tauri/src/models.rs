use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Search queries
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchQuery {
    pub id: String,
    pub query: String,
    pub tags: String, // JSON array of tags
    pub created_at: String,
    pub completed_at: Option<String>,
}

impl SearchQuery {
    pub fn new(query: String, tags: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            query,
            tags: serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string()),
            created_at: Utc::now().to_rfc3339(),
            completed_at: None,
        }
    }

    #[allow(dead_code)]
    pub fn get_tags(&self) -> Vec<String> {
        serde_json::from_str(&self.tags).unwrap_or_default()
    }
}

// Domains discovered from search
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Domain {
    pub id: String,
    pub search_query_id: String,
    pub domain: String,
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub crawl_status: String, // pending, in_progress, completed, failed
    pub crawl_started_at: Option<String>,
    pub crawl_completed_at: Option<String>,
    pub error_message: Option<String>,
    pub created_at: String,
}

impl Domain {
    pub fn new(search_query_id: String, domain: String, url: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            search_query_id,
            domain,
            url,
            title: None,
            description: None,
            crawl_status: "pending".to_string(),
            crawl_started_at: None,
            crawl_completed_at: None,
            error_message: None,
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

// Extracted emails
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Email {
    pub id: String,
    pub search_query_id: String,
    pub domain_id: String,
    pub domain: String,
    pub email: String,
    pub source_url: String,
    pub page_title: Option<String>,
    pub tags: String, // JSON array of tags from the search query
    pub found_at: String,
}

impl Email {
    pub fn new(
        search_query_id: String,
        domain_id: String,
        domain: String,
        email: String,
        source_url: String,
        page_title: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            search_query_id,
            domain_id,
            domain,
            email,
            source_url,
            page_title,
            tags: serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string()),
            found_at: Utc::now().to_rfc3339(),
        }
    }

    #[allow(dead_code)]
    pub fn get_tags(&self) -> Vec<String> {
        serde_json::from_str(&self.tags).unwrap_or_default()
    }
}

// Configuration - reserved for advanced config management in future versions
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Config {
    pub key: String,
    pub value: String,
}

// Search results from Google
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleSearchResult {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

// Crawl progress event - reserved for real-time progress tracking in future versions
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrawlProgress {
    pub domain_id: String,
    pub domain: String,
    pub status: String, // in_progress, completed, error
    pub pages_crawled: u32,
    pub emails_found: u32,
    pub current_url: Option<String>,
    pub error_message: Option<String>,
}

// Email list response with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailListResponse {
    pub emails: Vec<Email>,
    pub total: u32,
    pub page: u32,
    pub per_page: u32,
}

// Filter options - reserved for advanced email filtering in future versions
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailFilter {
    pub search_query_id: Option<String>,
    pub domain: Option<String>,
    pub tags: Vec<String>,
    pub search_term: Option<String>,
}

// Sort options - reserved for email sorting UI in future versions
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    #[serde(rename = "found_at_desc")]
    FoundAtDesc,
    #[serde(rename = "found_at_asc")]
    FoundAtAsc,
    #[serde(rename = "email_asc")]
    EmailAsc,
    #[serde(rename = "email_desc")]
    EmailDesc,
    #[serde(rename = "domain_asc")]
    DomainAsc,
    #[serde(rename = "domain_desc")]
    DomainDesc,
}

// Application state - holds database connection pool
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
}
