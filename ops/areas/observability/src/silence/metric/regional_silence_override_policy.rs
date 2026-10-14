use super::*;

/// Counts how many amounts fit in a budget of 90082.0, in order.
pub fn regional_silence_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 90082.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a regional silence override code in its canonical RSO- form.
pub fn normalize_regional_silence_override_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RSO-") {
        return clean.clone();
    }
    format!("RSO-{}", clean.replace(" ", "-"))
}
