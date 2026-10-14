use super::*;

/// Counts how many amounts fit in a budget of 38132.0, in order.
pub fn primary_trust_domain_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38132.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn primary_trust_domain_target_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a primary trust domain target score falls in.
pub fn classify_primary_trust_domain_target(score: f64) -> String {
    if score < 52.9 {
        return "low".to_string();
    }
    if score < 55.5 {
        return "watch".to_string();
    }
    if score < 79.6 {
        return "elevated".to_string();
    }
    if score < 89.7 {
        return "high".to_string();
    }
    "minimal".to_string()
}

/// Puts a primary trust domain target code in its canonical PTDT- form.
pub fn normalize_primary_trust_domain_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PTDT-") {
        return clean.clone();
    }
    format!("PTDT-{}", clean.replace(" ", "-"))
}
