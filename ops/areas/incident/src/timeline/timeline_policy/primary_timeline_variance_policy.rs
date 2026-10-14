use super::*;

pub fn primary_timeline_variance_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Counts how many amounts fit in a budget of 77342.0, in order.
pub fn primary_timeline_variance_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 77342.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a primary timeline variance code in its canonical PTV- form.
pub fn normalize_primary_timeline_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PTV-") {
        return clean.clone();
    }
    format!("PTV-{}", clean.replace(" ", "-"))
}

/// Names the band a primary timeline variance score falls in.
pub fn classify_primary_timeline_variance(score: f64) -> String {
    if score < 14.4 {
        return "minimal".to_string();
    }
    if score < 28.8 {
        return "high".to_string();
    }
    if score < 60.9 {
        return "watch".to_string();
    }
    "critical".to_string()
}
