use super::*;

/// Counts how many amounts fit in a budget of 38734.0, in order.
pub fn global_zone_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38734.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a global zone digest code in its canonical GZD- form.
pub fn normalize_global_zone_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GZD-") {
        return clean.clone();
    }
    format!("GZD-{}", clean.replace(" ", "-"))
}

pub fn global_zone_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a global zone digest score falls in.
pub fn classify_global_zone_digest(score: f64) -> String {
    if score < 25.0 {
        return "low".to_string();
    }
    if score < 48.6 {
        return "ok".to_string();
    }
    if score < 61.6 {
        return "minimal".to_string();
    }
    if score < 90.1 {
        return "critical".to_string();
    }
    "elevated".to_string()
}
