use serde_json::{json, Value};
use tauri::State;

use crate::db;
use crate::models::AppState;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ConfigUpdateRequest {
    pub google_api_key: Option<String>,
    pub google_search_engine_id: Option<String>,
    pub google_search_results: Option<u32>,
    pub max_pages_per_domain: Option<u32>,
    pub max_crawl_depth: Option<u32>,
    pub request_timeout_secs: Option<u32>,
    pub rate_limit_delay_ms: Option<u32>,
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Value, String> {
    let pool = &state.db;

    let mut config = json!({
        "google_api_key": "",
        "google_search_engine_id": "",
        "google_search_results": 10,
        "max_pages_per_domain": 100,
        "max_crawl_depth": 3,
        "request_timeout_secs": 10,
        "rate_limit_delay_ms": 500,
    });

    // Try to load from database
    if let Ok(Some(api_key)) = db::get_config(pool, "google_api_key").await {
        config["google_api_key"] = Value::String(api_key);
    }

    if let Ok(Some(engine_id)) = db::get_config(pool, "google_search_engine_id").await {
        config["google_search_engine_id"] = Value::String(engine_id);
    }

    if let Ok(Some(search_results)) = db::get_config(pool, "google_search_results").await {
        if let Ok(num) = search_results.parse::<u32>() {
            config["google_search_results"] = Value::Number(num.into());
        }
    }

    if let Ok(Some(max_pages)) = db::get_config(pool, "max_pages_per_domain").await {
        if let Ok(num) = max_pages.parse::<u32>() {
            config["max_pages_per_domain"] = Value::Number(num.into());
        }
    }

    if let Ok(Some(max_depth)) = db::get_config(pool, "max_crawl_depth").await {
        if let Ok(num) = max_depth.parse::<u32>() {
            config["max_crawl_depth"] = Value::Number(num.into());
        }
    }

    if let Ok(Some(timeout)) = db::get_config(pool, "request_timeout_secs").await {
        if let Ok(num) = timeout.parse::<u32>() {
            config["request_timeout_secs"] = Value::Number(num.into());
        }
    }

    if let Ok(Some(delay)) = db::get_config(pool, "rate_limit_delay_ms").await {
        if let Ok(num) = delay.parse::<u32>() {
            config["rate_limit_delay_ms"] = Value::Number(num.into());
        }
    }

    Ok(config)
}

#[tauri::command]
pub async fn set_config(
    state: State<'_, AppState>,
    config: ConfigUpdateRequest,
) -> Result<String, String> {
    let pool = &state.db;

    if let Some(api_key) = config.google_api_key {
        db::set_config(pool, "google_api_key", &api_key)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(engine_id) = config.google_search_engine_id {
        db::set_config(pool, "google_search_engine_id", &engine_id)
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(search_results) = config.google_search_results {
        db::set_config(pool, "google_search_results", &search_results.to_string())
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(max_pages) = config.max_pages_per_domain {
        db::set_config(pool, "max_pages_per_domain", &max_pages.to_string())
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(max_depth) = config.max_crawl_depth {
        db::set_config(pool, "max_crawl_depth", &max_depth.to_string())
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(timeout) = config.request_timeout_secs {
        db::set_config(pool, "request_timeout_secs", &timeout.to_string())
            .await
            .map_err(|e| e.to_string())?;
    }

    if let Some(delay) = config.rate_limit_delay_ms {
        db::set_config(pool, "rate_limit_delay_ms", &delay.to_string())
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok("Config updated successfully".to_string())
}
