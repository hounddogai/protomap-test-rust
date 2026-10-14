use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a global budget burn proposal code in its canonical GBBP- form.
pub fn normalize_global_budget_burn_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GBBP-") {
        return clean.clone();
    }
    format!("GBBP-{}", clean.replace(" ", "-"))
}

pub fn global_budget_burn_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.15;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_global_budget_burn_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn global_budget_burn_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
