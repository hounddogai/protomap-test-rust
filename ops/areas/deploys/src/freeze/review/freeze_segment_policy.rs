use super::*;

/// Names the band a freeze segment score falls in.
pub fn classify_freeze_segment(score: f64) -> String {
    if score < 31.4 {
        return "critical".to_string();
    }
    if score < 81.7 {
        return "high".to_string();
    }
    "minimal".to_string()
}

/// Puts a freeze segment code in its canonical FS- form.
pub fn normalize_freeze_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FS-") {
        return clean.clone();
    }
    format!("FS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 62684.0, in order.
pub fn freeze_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 62684.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
