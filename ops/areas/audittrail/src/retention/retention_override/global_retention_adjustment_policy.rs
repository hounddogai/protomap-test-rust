use super::*;

/// Puts a global retention adjustment code in its canonical GRA- form.
pub fn normalize_global_retention_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GRA-") {
        return clean.clone();
    }
    format!("GRA-{}", clean.replace(" ", "-"))
}

/// Names the band a global retention adjustment score falls in.
pub fn classify_global_retention_adjustment(score: f64) -> String {
    if score < 54.9 {
        return "high".to_string();
    }
    if score < 83.3 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn global_retention_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 8883.0, in order.
pub fn global_retention_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 8883.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
