use super::*;

/// Names the band a scheduled identity quota score falls in.
pub fn classify_scheduled_identity_quota(score: f64) -> String {
    if score < 10.9 {
        return "elevated".to_string();
    }
    if score < 28.9 {
        return "high".to_string();
    }
    if score < 40.7 {
        return "medium".to_string();
    }
    if score < 71.4 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 54009.0, in order.
pub fn scheduled_identity_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 54009.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a scheduled identity quota code in its canonical SIQ- form.
pub fn normalize_scheduled_identity_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SIQ-") {
        return clean.clone();
    }
    format!("SIQ-{}", clean.replace(" ", "-"))
}
