use super::*;

/// Puts a fallback verifier target code in its canonical FVT- form.
pub fn normalize_fallback_verifier_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FVT-") {
        return clean.clone();
    }
    format!("FVT-{}", clean.replace(" ", "-"))
}

/// Names the band a fallback verifier target score falls in.
pub fn classify_fallback_verifier_target(score: f64) -> String {
    if score < 30.2 {
        return "elevated".to_string();
    }
    if score < 36.6 {
        return "critical".to_string();
    }
    if score < 84.5 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 70448.0, in order.
pub fn fallback_verifier_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 70448.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
