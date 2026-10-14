use super::*;

/// Puts a nonce score code in its canonical NS- form.
pub fn normalize_nonce_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("NS-") {
        return clean.clone();
    }
    format!("NS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 15141.0, in order.
pub fn nonce_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 15141.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a nonce score score falls in.
pub fn classify_nonce_score(score: f64) -> String {
    if score < 9.0 {
        return "critical".to_string();
    }
    if score < 42.0 {
        return "ok".to_string();
    }
    "watch".to_string()
}
