use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a budget burn rule code in its canonical BBR- form.
pub fn normalize_budget_burn_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBR-") {
        return clean.clone();
    }
    format!("BBR-{}", clean.replace(" ", "-"))
}

pub fn merge_budget_burn_rule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a budget burn rule score falls in.
pub fn classify_budget_burn_rule(score: f64) -> String {
    if score < 37.0 {
        return "minimal".to_string();
    }
    if score < 50.2 {
        return "watch".to_string();
    }
    if score < 60.5 {
        return "elevated".to_string();
    }
    "high".to_string()
}
