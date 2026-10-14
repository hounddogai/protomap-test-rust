use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn manual_dns_record_reservation_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a manual dns record reservation code in its canonical MDRR- form.
pub fn normalize_manual_dns_record_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MDRR-") {
        return clean.clone();
    }
    format!("MDRR-{}", clean.replace(" ", "-"))
}

pub fn merge_manual_dns_record_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
