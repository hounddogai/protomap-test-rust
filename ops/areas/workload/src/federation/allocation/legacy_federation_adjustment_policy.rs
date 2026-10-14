use super::*;

/// Puts a legacy federation adjustment code in its canonical LFA- form.
pub fn normalize_legacy_federation_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LFA-") {
        return clean.clone();
    }
    format!("LFA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 1171.0, in order.
pub fn legacy_federation_adjustment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1171.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a legacy federation adjustment score falls in.
pub fn classify_legacy_federation_adjustment(score: f64) -> String {
    if score < 15.1 {
        return "medium".to_string();
    }
    if score < 25.7 {
        return "ok".to_string();
    }
    if score < 68.2 {
        return "low".to_string();
    }
    if score < 81.4 {
        return "critical".to_string();
    }
    "watch".to_string()
}
