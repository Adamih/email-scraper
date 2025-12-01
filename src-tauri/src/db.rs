use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::path::Path;
use std::str::FromStr;

use crate::models::{Domain, Email, SearchQuery};

pub async fn init_db(db_path: &Path) -> Result<SqlitePool, sqlx::Error> {
    // Ensure the directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // Create database URL with proper SQLite format
    let db_path_str = db_path.to_string_lossy().to_string();
    let database_url = if db_path_str.starts_with('/') {
        format!("sqlite://{}", db_path_str)
    } else {
        format!("sqlite:{}", db_path_str)
    };

    let connect_options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Create tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS search_queries (
            id TEXT PRIMARY KEY,
            query TEXT NOT NULL,
            tags TEXT NOT NULL,
            created_at TEXT NOT NULL,
            completed_at TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS domains (
            id TEXT PRIMARY KEY,
            search_query_id TEXT NOT NULL,
            domain TEXT NOT NULL,
            url TEXT NOT NULL,
            title TEXT,
            description TEXT,
            crawl_status TEXT NOT NULL,
            crawl_started_at TEXT,
            crawl_completed_at TEXT,
            error_message TEXT,
            created_at TEXT NOT NULL,
            FOREIGN KEY (search_query_id) REFERENCES search_queries(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS emails (
            id TEXT PRIMARY KEY,
            search_query_id TEXT NOT NULL,
            domain_id TEXT NOT NULL,
            domain TEXT NOT NULL,
            email TEXT NOT NULL,
            source_url TEXT NOT NULL,
            page_title TEXT,
            tags TEXT NOT NULL,
            found_at TEXT NOT NULL,
            FOREIGN KEY (search_query_id) REFERENCES search_queries(id),
            FOREIGN KEY (domain_id) REFERENCES domains(id)
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS config (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    // Create indexes
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_domains_search_query_id ON domains(search_query_id)",
    )
    .execute(&pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_emails_search_query_id ON emails(search_query_id)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_emails_domain_id ON emails(domain_id)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_emails_domain ON emails(domain)")
        .execute(&pool)
        .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_emails_email ON emails(email)")
        .execute(&pool)
        .await?;

    Ok(pool)
}

// Search Query operations
#[allow(dead_code)]
pub async fn insert_search_query(
    pool: &SqlitePool,
    search_query: &SearchQuery,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO search_queries (id, query, tags, created_at, completed_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&search_query.id)
    .bind(&search_query.query)
    .bind(&search_query.tags)
    .bind(&search_query.created_at)
    .bind(&search_query.completed_at)
    .execute(pool)
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn get_search_query(
    pool: &SqlitePool,
    id: &str,
) -> Result<Option<SearchQuery>, sqlx::Error> {
    sqlx::query_as::<_, SearchQuery>("SELECT * FROM search_queries WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
}

#[allow(dead_code)]
pub async fn get_all_search_queries(pool: &SqlitePool) -> Result<Vec<SearchQuery>, sqlx::Error> {
    sqlx::query_as::<_, SearchQuery>("SELECT * FROM search_queries ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

#[allow(dead_code)]
pub async fn update_search_query_completed(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("UPDATE search_queries SET completed_at = ? WHERE id = ?")
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

// Domain operations
pub async fn insert_domain(pool: &SqlitePool, domain: &Domain) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO domains (id, search_query_id, domain, url, title, description, crawl_status, crawl_started_at, crawl_completed_at, error_message, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&domain.id)
    .bind(&domain.search_query_id)
    .bind(&domain.domain)
    .bind(&domain.url)
    .bind(&domain.title)
    .bind(&domain.description)
    .bind(&domain.crawl_status)
    .bind(&domain.crawl_started_at)
    .bind(&domain.crawl_completed_at)
    .bind(&domain.error_message)
    .bind(&domain.created_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_domains_by_search_query(
    pool: &SqlitePool,
    search_query_id: &str,
) -> Result<Vec<Domain>, sqlx::Error> {
    sqlx::query_as::<_, Domain>(
        "SELECT * FROM domains WHERE search_query_id = ? ORDER BY created_at DESC",
    )
    .bind(search_query_id)
    .fetch_all(pool)
    .await
}

pub async fn get_pending_domains(pool: &SqlitePool) -> Result<Vec<Domain>, sqlx::Error> {
    sqlx::query_as::<_, Domain>(
        "SELECT * FROM domains WHERE crawl_status = 'pending' ORDER BY created_at ASC",
    )
    .fetch_all(pool)
    .await
}

pub async fn update_domain_status(
    pool: &SqlitePool,
    domain_id: &str,
    status: &str,
    error_message: Option<String>,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now().to_rfc3339();

    if status == "in_progress" {
        sqlx::query("UPDATE domains SET crawl_status = ?, crawl_started_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(domain_id)
            .execute(pool)
            .await?;
    } else if status == "completed" {
        sqlx::query("UPDATE domains SET crawl_status = ?, crawl_completed_at = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(domain_id)
            .execute(pool)
            .await?;
    } else if status == "failed" {
        sqlx::query("UPDATE domains SET crawl_status = ?, crawl_completed_at = ?, error_message = ? WHERE id = ?")
            .bind(status)
            .bind(&now)
            .bind(error_message)
            .bind(domain_id)
            .execute(pool)
            .await?;
    }

    Ok(())
}

// Email operations
pub async fn insert_email(pool: &SqlitePool, email: &Email) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO emails (id, search_query_id, domain_id, domain, email, source_url, page_title, tags, found_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&email.id)
    .bind(&email.search_query_id)
    .bind(&email.domain_id)
    .bind(&email.domain)
    .bind(&email.email)
    .bind(&email.source_url)
    .bind(&email.page_title)
    .bind(&email.tags)
    .bind(&email.found_at)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn email_exists(
    pool: &SqlitePool,
    email: &str,
    domain_id: &str,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("SELECT COUNT(*) as count FROM emails WHERE email = ? AND domain_id = ?")
            .bind(email)
            .bind(domain_id)
            .fetch_one(pool)
            .await?;

    let count: i64 = result.get("count");
    Ok(count > 0)
}

pub async fn get_emails_paginated(
    pool: &SqlitePool,
    page: u32,
    per_page: u32,
) -> Result<(Vec<Email>, u32), sqlx::Error> {
    let offset = (page - 1) * per_page;

    let total: u32 = sqlx::query("SELECT COUNT(*) as count FROM emails")
        .fetch_one(pool)
        .await?
        .get("count");

    let emails =
        sqlx::query_as::<_, Email>("SELECT * FROM emails ORDER BY found_at DESC LIMIT ? OFFSET ?")
            .bind(per_page)
            .bind(offset)
            .fetch_all(pool)
            .await?;

    Ok((emails, total))
}

pub async fn get_emails_by_search_query(
    pool: &SqlitePool,
    search_query_id: &str,
    page: u32,
    per_page: u32,
) -> Result<(Vec<Email>, u32), sqlx::Error> {
    let offset = (page - 1) * per_page;

    let total: u32 = sqlx::query("SELECT COUNT(*) as count FROM emails WHERE search_query_id = ?")
        .bind(search_query_id)
        .fetch_one(pool)
        .await?
        .get("count");

    let emails = sqlx::query_as::<_, Email>(
        "SELECT * FROM emails WHERE search_query_id = ? ORDER BY found_at DESC LIMIT ? OFFSET ?",
    )
    .bind(search_query_id)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((emails, total))
}

pub async fn get_emails_by_domain(
    pool: &SqlitePool,
    domain: &str,
    page: u32,
    per_page: u32,
) -> Result<(Vec<Email>, u32), sqlx::Error> {
    let offset = (page - 1) * per_page;

    let total: u32 = sqlx::query("SELECT COUNT(*) as count FROM emails WHERE domain = ?")
        .bind(domain)
        .fetch_one(pool)
        .await?
        .get("count");

    let emails = sqlx::query_as::<_, Email>(
        "SELECT * FROM emails WHERE domain = ? ORDER BY found_at DESC LIMIT ? OFFSET ?",
    )
    .bind(domain)
    .bind(per_page)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok((emails, total))
}

pub async fn get_emails_count_by_domain(
    pool: &SqlitePool,
) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let results = sqlx::query(
        "SELECT domain, COUNT(*) as count FROM emails GROUP BY domain ORDER BY count DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(results
        .iter()
        .map(|row| (row.get::<String, _>("domain"), row.get::<i64, _>("count")))
        .collect())
}

// Config operations
pub async fn get_config(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let result = sqlx::query("SELECT value FROM config WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;

    Ok(result.map(|row| row.get("value")))
}

pub async fn set_config(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR REPLACE INTO config (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;

    Ok(())
}

// Delete operations
pub async fn delete_search_query(pool: &SqlitePool, search_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM search_queries WHERE id = ?")
        .bind(search_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_domains_by_search(
    pool: &SqlitePool,
    search_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM domains WHERE search_query_id = ?")
        .bind(search_id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn delete_emails_by_search(
    pool: &SqlitePool,
    search_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM emails WHERE search_query_id = ?")
        .bind(search_id)
        .execute(pool)
        .await?;

    Ok(())
}
