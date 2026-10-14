use super::*;

/// Names the band a sink quota score falls in.
pub fn classify_sink_quota(score: f64) -> String {
    if score < 30.7 {
        return "high".to_string();
    }
    if score < 54.3 {
        return "low".to_string();
    }
    if score < 65.8 {
        return "elevated".to_string();
    }
    if score < 91.8 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Puts a sink quota code in its canonical SQ- form.
pub fn normalize_sink_quota_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SQ-") {
        return clean.clone();
    }
    format!("SQ-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 16398.0, in order.
pub fn sink_quota_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 16398.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
