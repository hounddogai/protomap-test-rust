use super::*;

pub fn segment_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Names the band a segment checkpoint score falls in.
pub fn classify_segment_checkpoint(score: f64) -> String {
    if score < 10.6 {
        return "high".to_string();
    }
    if score < 67.6 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Puts a segment checkpoint code in its canonical SC- form.
pub fn normalize_segment_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SC-") {
        return clean.clone();
    }
    format!("SC-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 75181.0, in order.
pub fn segment_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 75181.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
