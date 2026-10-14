use super::*;

/// Puts a lenient canary slot code in its canonical LCS- form.
pub fn normalize_lenient_canary_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LCS-") {
        return clean.clone();
    }
    format!("LCS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 4242.0, in order.
pub fn lenient_canary_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 4242.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a lenient canary slot score falls in.
pub fn classify_lenient_canary_slot(score: f64) -> String {
    if score < 12.2 {
        return "minimal".to_string();
    }
    if score < 13.7 {
        return "critical".to_string();
    }
    if score < 58.8 {
        return "medium".to_string();
    }
    "ok".to_string()
}
