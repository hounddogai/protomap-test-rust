use super::*;

/// Names the band a timeline profile score falls in.
pub fn classify_timeline_profile(score: f64) -> String {
    if score < 3.6 {
        return "watch".to_string();
    }
    if score < 67.8 {
        return "elevated".to_string();
    }
    "high".to_string()
}

/// Counts how many amounts fit in a budget of 78983.0, in order.
pub fn timeline_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 78983.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn timeline_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Puts a timeline profile code in its canonical TP- form.
pub fn normalize_timeline_profile_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TP-") {
        return clean.clone();
    }
    format!("TP-{}", clean.replace(" ", "-"))
}
