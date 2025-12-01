use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashSet;

lazy_static! {
    // Simple but effective email regex pattern
    static ref EMAIL_REGEX: Regex = Regex::new(
        r#"[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*"#
    ).unwrap();

    // Common dummy/placeholder email patterns that should be ignored
    static ref DUMMY_EMAIL_IGNORE_LIST: HashSet<&'static str> = {
        vec![
            // Generic placeholders
            "user@domain.com",
            "test@example.com",
            "admin@example.com",
            "info@example.com",
            "support@example.com",
            "hello@example.com",
            "contact@example.com",
            "mail@example.com",
            "email@example.com",
            "example@example.com",

            // Common test domains
            "user@test.com",
            "admin@test.com",
            "test@test.com",
            "demo@demo.com",
            "user@demo.com",
            "contact@demo.org",

            // Placeholder patterns
            "yourname@example.com",
            "yourname@yourdomain.com",
            "name@example.com",
            "person@example.com",
            "someone@example.com",

            // Default/sample emails
            "sample@example.com",
            "example@domain.com",
            "dummy@example.com",
            "placeholder@example.com",
            "fake@example.com",

            // Common local test addresses
            "webmaster@example.com",
            "postmaster@example.com",
            "root@localhost",
            "admin@localhost",

            // Common noreply addresses (often dummy)
            "noreply@example.com",
            "no-reply@example.com",
            "donotreply@example.com",
            "do-not-reply@example.com",
        ]
        .into_iter()
        .collect()
    };
}

/// Extract email addresses from HTML content
pub fn extract_emails(html_content: &str) -> Vec<String> {
    let mut emails = Vec::new();
    let mut seen = HashSet::new();

    for cap in EMAIL_REGEX.captures_iter(html_content) {
        if let Some(email_match) = cap.get(0) {
            let email = email_match.as_str().to_lowercase();
            if is_valid_email(&email) && !seen.contains(&email) {
                emails.push(email.clone());
                seen.insert(email);
            }
        }
    }

    emails
}

/// Validate email address with additional checks
fn is_valid_email(email: &str) -> bool {
    // Basic checks
    if email.len() > 254 {
        return false;
    }

    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return false;
    }

    let local_part = parts[0];
    let domain_part = parts[1];

    // Validate local part
    if local_part.is_empty() || local_part.len() > 64 {
        return false;
    }

    // Check for invalid patterns
    if local_part.starts_with('.') || local_part.ends_with('.') || local_part.contains("..") {
        return false;
    }

    // Validate domain part
    if domain_part.is_empty() {
        return false;
    }

    if !domain_part.contains('.') {
        return false;
    }

    // Domain shouldn't start or end with a dot
    if domain_part.starts_with('.') || domain_part.ends_with('.') {
        return false;
    }

    // Check domain TLD
    let domain_parts: Vec<&str> = domain_part.split('.').collect();
    let tld = domain_parts.last().unwrap_or(&"");

    // TLD must be at least 2 characters
    if tld.len() < 2 {
        return false;
    }

    // Filter out common non-email patterns
    if is_probable_non_email(email) {
        return false;
    }

    true
}

/// Filter out common false positives
fn is_probable_non_email(email: &str) -> bool {
    let lower = email.to_lowercase();

    // Check against ignore list of common dummy emails
    if DUMMY_EMAIL_IGNORE_LIST.contains(lower.as_str()) {
        return true;
    }

    // Common non-email patterns
    if lower.ends_with(".com.") || lower.ends_with(".org.") || lower.ends_with(".net.") {
        return true;
    }

    // Version numbers
    if lower.contains("version") || lower.contains("v.") {
        return true;
    }

    // Common fake/test emails
    if lower.contains("example") || lower.contains("test") || lower.contains("demo") {
        return true;
    }

    // File extensions that might be caught
    if lower.ends_with(".html") || lower.ends_with(".png") || lower.ends_with(".jpg") {
        return true;
    }

    // Error tracking and logging services (Sentry, error tracking, etc)
    if lower.contains("sentry")
        || lower.contains("error-tracking")
        || lower.contains("error_tracking")
        || lower.contains("ingest.")
        || lower.contains("tracking@")
    {
        return true;
    }

    // URLs that got caught as emails (start with //)
    if lower.starts_with("//") {
        return true;
    }

    // Localhost and internal addresses
    if lower.contains("localhost")
        || lower.contains("127.0.0.1")
        || lower.contains("192.168")
        || lower.contains("10.0.")
        || lower.contains("172.16.")
    {
        return true;
    }

    // Common internal service patterns
    if lower.contains("@localhost")
        || lower.contains("@internal")
        || lower.contains("@corp")
        || lower.contains("@dev")
        || lower.contains("noreply")
        || lower.contains("no-reply")
        || lower.contains("donotreply")
        || lower.contains("do-not-reply")
    {
        return true;
    }

    // Analytics and monitoring services
    if lower.contains("analytics")
        || lower.contains("tracking")
        || lower.contains("google-analytics")
        || lower.contains("segment")
        || lower.contains("mixpanel")
        || lower.contains("amplitude")
        || lower.contains("datadog")
    {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        // Use real-looking emails that aren't in the ignore list
        assert!(is_valid_email("john@acme.com"));
        assert!(is_valid_email("user.name@company.org"));
        assert!(is_valid_email("alice+tag@techcorp.co.uk"));
    }

    #[test]
    fn test_invalid_emails() {
        assert!(!is_valid_email("invalid.email@"));
        assert!(!is_valid_email("@example.com"));
        assert!(!is_valid_email("invalid@.com"));
        assert!(!is_valid_email("invalid..email@example.com"));
    }

    #[test]
    fn test_filter_error_tracking_emails() {
        // Error tracking and logging services should be filtered
        assert!(!is_valid_email(
            "9f057df6115a4bb488c08ea12a835e6e@error-tracking.reddit.com"
        ));
        assert!(!is_valid_email(
            "9f057df6115a4bb488c08ea12a835e6e@o418887.ingest.sentry.io"
        ));
        assert!(!is_valid_email("analytics@google-analytics.com"));
        assert!(!is_valid_email("noreply@example.com"));
    }

    #[test]
    fn test_filter_internal_services() {
        // Internal service emails should be filtered
        assert!(!is_valid_email("dev@localhost"));
        assert!(!is_valid_email("admin@internal.com"));
        assert!(!is_valid_email("tracking@segment.io"));
    }

    #[test]
    fn test_filter_dummy_emails() {
        // Common placeholder emails should be filtered
        assert!(!is_valid_email("user@domain.com"));
        assert!(!is_valid_email("test@example.com"));
        assert!(!is_valid_email("admin@example.com"));
        assert!(!is_valid_email("info@example.com"));
        assert!(!is_valid_email("support@example.com"));
        assert!(!is_valid_email("contact@demo.org"));
        assert!(!is_valid_email("sample@example.com"));
        assert!(!is_valid_email("webmaster@example.com"));
        assert!(!is_valid_email("postmaster@example.com"));
    }

    #[test]
    fn test_extract_emails() {
        let html = r#"
            Contact us at john@acme.com or sarah@techcorp.org
            You can also reach mike@innovate.io
        "#;
        let emails = extract_emails(html);
        assert_eq!(emails.len(), 3);
    }
}
