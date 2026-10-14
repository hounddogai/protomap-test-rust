use super::*;

/// Counts how many amounts fit in a budget of 78674.0, in order.
pub fn fallback_page_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 78674.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a fallback page adjustment score falls in.
pub fn classify_fallback_page_adjustment(score: f64) -> String {
    if score < 2.5 {
        return "high".to_string();
    }
    if score < 66.2 {
        return "minimal".to_string();
    }
    if score < 88.6 {
        return "low".to_string();
    }
    "elevated".to_string()
}

pub fn fallback_page_adjustment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Puts a fallback page adjustment code in its canonical FPA- form.
pub fn normalize_fallback_page_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FPA-") {
        return clean.clone();
    }
    format!("FPA-{}", clean.replace(" ", "-"))
}
