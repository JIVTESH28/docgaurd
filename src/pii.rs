use regex::Regex;
use std::sync::OnceLock;

// Static compile-once regex registry for maximum performance
static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
static PHONE_REGEX: OnceLock<Regex> = OnceLock::new();
static SSN_REGEX: OnceLock<Regex> = OnceLock::new();
static IP_REGEX: OnceLock<Regex> = OnceLock::new();
static CREDIT_CARD_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_email_regex() -> &'static Regex {
    EMAIL_REGEX.get_or_init(|| Regex::new(r"(?i)[a-z0-9_.+-]+@[a-z0-9-]+\.[a-z0-9-.]+").unwrap())
}

fn get_phone_regex() -> &'static Regex {
    // Matches +1-234-567-8901, (123) 456-7890, 123-456-7890, 123.456.7890
    PHONE_REGEX.get_or_init(|| Regex::new(r"(?:\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}").unwrap())
}

fn get_ssn_regex() -> &'static Regex {
    SSN_REGEX.get_or_init(|| Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap())
}

fn get_ip_regex() -> &'static Regex {
    // Matches IPv4 addresses
    IP_REGEX.get_or_init(|| Regex::new(r"\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b").unwrap())
}

fn get_credit_card_regex() -> &'static Regex {
    // Matches 13 to 16 digit credit cards with optional hyphens or spaces
    CREDIT_CARD_REGEX.get_or_init(|| Regex::new(r"\b(?:\d[ -]*?){13,16}\b").unwrap())
}

/// Redacts sensitive PII entities from the input text.
///
/// Supported entities: "email", "phone", "ssn", "ip", "credit_card".
/// If `entities` slice is empty, all categories will be redacted by default.
pub fn redact_pii(text: &str, entities: &[String]) -> String {
    let mut redacted = text.to_string();
    let all = entities.is_empty();

    if all || entities.contains(&"email".to_string()) {
        redacted = get_email_regex().replace_all(&redacted, "[EMAIL]").into_owned();
    }

    if all || entities.contains(&"phone".to_string()) {
        redacted = get_phone_regex().replace_all(&redacted, "[PHONE]").into_owned();
    }

    if all || entities.contains(&"ssn".to_string()) {
        redacted = get_ssn_regex().replace_all(&redacted, "[SSN]").into_owned();
    }

    if all || entities.contains(&"ip".to_string()) {
        redacted = get_ip_regex().replace_all(&redacted, "[IP_ADDRESS]").into_owned();
    }

    if all || entities.contains(&"credit_card".to_string()) {
        redacted = get_credit_card_regex().replace_all(&redacted, "[CREDIT_CARD]").into_owned();
    }

    redacted
}

/// Scans the text and returns a list of detected PII categories.
pub fn detect_pii(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    if get_email_regex().is_match(text) {
        found.push("email".to_string());
    }
    if get_phone_regex().is_match(text) {
        found.push("phone".to_string());
    }
    if get_ssn_regex().is_match(text) {
        found.push("ssn".to_string());
    }
    if get_ip_regex().is_match(text) {
        found.push("ip".to_string());
    }
    if get_credit_card_regex().is_match(text) {
        found.push("credit_card".to_string());
    }
    found
}
