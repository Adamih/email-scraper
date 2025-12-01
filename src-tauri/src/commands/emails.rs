use tauri::State;

use crate::db;
use crate::models::{AppState, EmailListResponse};

#[tauri::command]
pub async fn get_emails(
    state: State<'_, AppState>,
    page: u32,
    per_page: u32,
) -> Result<EmailListResponse, String> {
    let pool = &state.db;
    let (emails, total) = db::get_emails_paginated(pool, page, per_page)
        .await
        .map_err(|e| e.to_string())?;

    Ok(EmailListResponse {
        emails,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub async fn get_emails_by_query(
    state: State<'_, AppState>,
    search_query_id: String,
    page: u32,
    per_page: u32,
) -> Result<EmailListResponse, String> {
    let pool = &state.db;
    let (emails, total) = db::get_emails_by_search_query(pool, &search_query_id, page, per_page)
        .await
        .map_err(|e| e.to_string())?;

    Ok(EmailListResponse {
        emails,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub async fn get_emails_by_domain(
    state: State<'_, AppState>,
    domain: String,
    page: u32,
    per_page: u32,
) -> Result<EmailListResponse, String> {
    let pool = &state.db;
    let (emails, total) = db::get_emails_by_domain(pool, &domain, page, per_page)
        .await
        .map_err(|e| e.to_string())?;

    Ok(EmailListResponse {
        emails,
        total,
        page,
        per_page,
    })
}

#[tauri::command]
pub async fn get_domain_statistics(
    state: State<'_, AppState>,
) -> Result<Vec<(String, i64)>, String> {
    let pool = &state.db;
    db::get_emails_count_by_domain(pool)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_to_csv(state: State<'_, AppState>) -> Result<String, String> {
    let pool = &state.db;

    // Fetch all emails from the database
    let (emails, _total) = db::get_emails_paginated(pool, 1, 1000000)
        .await
        .map_err(|e| format!("Failed to fetch emails: {}", e))?;

    // Build CSV content
    let mut csv = String::from("Email,Domain,Source URL,Page Title,Found At\n");

    for email in emails {
        let page_title = email.page_title.unwrap_or_default();
        let found_at = email.found_at;

        // Escape quotes in fields and wrap in quotes
        let email_addr = email.email.replace("\"", "\"\"");
        let domain = email.domain.replace("\"", "\"\"");
        let source_url = email.source_url.replace("\"", "\"\"");
        let page_title_escaped = page_title.replace("\"", "\"\"");

        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            email_addr, domain, source_url, page_title_escaped, found_at
        ));
    }

    // Generate filename with timestamp
    let now = chrono::Local::now();
    let timestamp = now.format("%Y%m%d_%H%M%S").to_string();
    let filename = format!("emails_{}.csv", timestamp);

    // Try to get Downloads directory, fallback to home directory
    let downloads_path = if let Ok(home) = std::env::var("HOME") {
        format!("{}/Downloads/{}", home, filename)
    } else {
        filename.clone()
    };

    // Write to file
    std::fs::write(&downloads_path, csv).map_err(|e| format!("Failed to write CSV file: {}", e))?;

    Ok(format!("CSV exported successfully to {}", downloads_path))
}
