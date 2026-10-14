use super::*;

/// Puts a credential quota code in its canonical CQ- form.
pub fn normalize_credential_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CQ-") {
        return clean.clone();
    }
    format!("CQ-{}", clean.replace(" ", "-"))
}

pub fn credential_quota_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 26954.0, in order.
pub fn credential_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 26954.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a credential quota score falls in.
pub fn classify_credential_quota(score: f64) -> String {
    if score < 10.6 {
        return "medium".to_string();
    }
    if score < 17.2 {
        return "critical".to_string();
    }
    if score < 47.3 {
        return "ok".to_string();
    }
    if score < 84.8 {
        return "low".to_string();
    }
    "elevated".to_string()
}
