use std::time::Duration;
use tauri::State;
use tokio::time::sleep;

use crate::db;
use crate::email_extractor;
use crate::models::{AppState, Domain, Email};

#[tauri::command]
pub async fn start_crawler(
    state: State<'_, AppState>,
    domain_id: String,
) -> Result<String, String> {
    let pool = state.db.clone();

    // Spawn crawler in background
    tokio::spawn(async move {
        let _ = crawl_domain(&pool, &domain_id).await;
    });

    Ok("Crawler started".to_string())
}

#[tauri::command]
pub async fn get_pending_domains(state: State<'_, AppState>) -> Result<Vec<Domain>, String> {
    let pool = &state.db;
    db::get_pending_domains(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_all_crawlers(state: State<'_, AppState>) -> Result<String, String> {
    let pool = state.db.clone();

    // Get all pending domains
    let pending_domains = db::get_pending_domains(&pool)
        .await
        .map_err(|e| e.to_string())?;

    let count = pending_domains.len();

    // Spawn all crawlers in parallel (non-blocking)
    for domain in pending_domains {
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            let _ = crawl_domain(&pool_clone, &domain.id).await;
        });
    }

    Ok(format!("Started crawling {} domains in parallel", count))
}

#[tauri::command]
pub async fn delete_domain(
    state: State<'_, AppState>,
    domain_id: String,
) -> Result<String, String> {
    let pool = &state.db;

    // Delete emails for this domain first (foreign key constraint)
    sqlx::query("DELETE FROM emails WHERE domain_id = ?")
        .bind(&domain_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    // Delete the domain
    sqlx::query("DELETE FROM domains WHERE id = ?")
        .bind(&domain_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok("Domain deleted successfully".to_string())
}

#[tauri::command]
pub async fn delete_all_pending_domains(state: State<'_, AppState>) -> Result<String, String> {
    let pool = &state.db;

    // Get all pending domain IDs
    let pending_domain_ids: Vec<(String,)> =
        sqlx::query_as("SELECT id FROM domains WHERE crawl_status = 'pending'")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

    let count = pending_domain_ids.len();

    // Delete emails for all pending domains
    for (domain_id,) in &pending_domain_ids {
        sqlx::query("DELETE FROM emails WHERE domain_id = ?")
            .bind(domain_id)
            .execute(pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    // Delete all pending domains
    sqlx::query("DELETE FROM domains WHERE crawl_status = 'pending'")
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!("Deleted {} pending domains", count))
}

async fn crawl_domain(pool: &sqlx::SqlitePool, domain_id: &str) -> Result<(), String> {
    // Get domain record
    let domain = sqlx::query_as::<_, Domain>("SELECT * FROM domains WHERE id = ?")
        .bind(domain_id)
        .fetch_one(pool)
        .await
        .map_err(|e| e.to_string())?;

    // Update status
    db::update_domain_status(pool, domain_id, "in_progress", None)
        .await
        .map_err(|e| e.to_string())?;

    log::info!("Starting crawl of domain: {}", domain.domain);

    // Get configuration
    let max_pages: u32 = db::get_config(pool, "max_pages_per_domain")
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    let rate_limit_ms: u64 = db::get_config(pool, "rate_limit_delay_ms")
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    // Get sitemap URLs
    let sitemap_urls = fetch_sitemap(&domain.domain, max_pages).await;

    let mut urls_to_crawl = vec![];

    // Always add the search result URL first (highest priority)
    urls_to_crawl.push(domain.url.clone());

    // Then add sitemap URLs (if any)
    for url in sitemap_urls {
        if url != domain.url && urls_to_crawl.len() < max_pages as usize {
            urls_to_crawl.push(url);
        }
    }

    // If we only have the search result URL and it's the only one, also try the domain root
    if urls_to_crawl.len() == 1 {
        urls_to_crawl.push(format!("https://{}", domain.domain));
    }

    // Limit number of pages
    urls_to_crawl.truncate(max_pages as usize);

    let mut emails_found = 0;
    let mut pages_crawled = 0;

    // Crawl each URL
    for url in urls_to_crawl {
        if pages_crawled >= max_pages {
            break;
        }

        match crawl_page(&url).await {
            Ok(html) => {
                pages_crawled += 1;

                // Extract emails
                let emails = email_extractor::extract_emails(&html);

                // Get page title
                let page_title = extract_page_title(&html);

                // Store emails
                for email in emails {
                    // Check if email already exists
                    if !db::email_exists(pool, &email, domain_id)
                        .await
                        .unwrap_or(false)
                    {
                        let email_record = Email::new(
                            domain.search_query_id.clone(),
                            domain_id.to_string(),
                            domain.domain.clone(),
                            email,
                            url.clone(),
                            page_title.clone(),
                            serde_json::from_str(&domain.search_query_id).unwrap_or_default(),
                        );

                        if db::insert_email(pool, &email_record).await.is_ok() {
                            emails_found += 1;
                        }
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to crawl {}: {}", url, e);
            }
        }

        // Rate limiting
        sleep(Duration::from_millis(rate_limit_ms)).await;
    }

    // Update domain status to completed
    db::update_domain_status(pool, domain_id, "completed", None)
        .await
        .map_err(|e| e.to_string())?;

    log::info!(
        "Completed crawl of {}: {} pages, {} emails found",
        domain.domain,
        pages_crawled,
        emails_found
    );

    Ok(())
}

async fn fetch_sitemap(domain: &str, max_urls: u32) -> Vec<String> {
    let mut urls = Vec::new();

    // List of common sitemap locations to try
    let mut sitemap_locations = vec![
        format!("https://{}/sitemap.xml", domain),
        format!("https://{}/sitemap_index.xml", domain),
        format!("https://www.{}/sitemap.xml", domain),
        format!("https://www.{}/sitemap_index.xml", domain),
        format!("https://{}/sitemaps.xml", domain),
    ];

    // Try to fetch robots.txt and extract sitemap URLs from it
    if let Ok(robots_content) = crawl_page(&format!("https://{}/robots.txt", domain)).await {
        // Extract sitemap URLs from robots.txt (Sitemap: https://example.com/sitemap.xml)
        let re = regex::Regex::new(r"(?i)Sitemap:\s*(.+)").unwrap();
        for cap in re.captures_iter(&robots_content) {
            if let Some(url_match) = cap.get(1) {
                let sitemap_url = url_match.as_str().trim().to_string();
                if !sitemap_url.is_empty() && !sitemap_locations.contains(&sitemap_url) {
                    sitemap_locations.push(sitemap_url);
                }
            }
        }
    }

    // Try each sitemap location
    for sitemap_url in sitemap_locations {
        if let Ok(content) = crawl_page(&sitemap_url).await {
            // Try to parse as XML sitemap
            let re = regex::Regex::new(r"<loc>([^<]+)</loc>").unwrap();
            let mut found_urls = false;

            for cap in re.captures_iter(&content) {
                if let Some(url_match) = cap.get(1) {
                    let url = url_match.as_str().to_string();
                    // Filter to same domain (handle www variations)
                    if url.contains(domain) || url.contains(&format!("www.{}", domain)) {
                        urls.push(url);
                        found_urls = true;
                    }
                }
            }

            // If we found URLs in this sitemap, stop searching
            if found_urls {
                log::info!("Found sitemap at: {}", sitemap_url);
                break;
            }
        }
    }

    // Remove duplicates
    urls.sort();
    urls.dedup();

    urls.truncate(max_urls as usize);
    urls
}

async fn crawl_page(url: &str) -> Result<String, String> {
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .header("User-Agent", "EmailScraper/1.0")
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let html = response.text().await.map_err(|e| e.to_string())?;
    Ok(html)
}

fn extract_page_title(html: &str) -> Option<String> {
    let re = regex::Regex::new(r"<title>([^<]+)</title>").ok()?;
    re.captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}
