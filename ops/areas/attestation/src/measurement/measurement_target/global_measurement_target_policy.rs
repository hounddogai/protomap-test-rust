use super::*;

/// Counts how many amounts fit in a budget of 94720.0, in order.
pub fn global_measurement_target_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 94720.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a global measurement target code in its canonical GMT- form.
pub fn normalize_global_measurement_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GMT-") {
        return clean.clone();
    }
    format!("GMT-{}", clean.replace(" ", "-"))
}

/// Names the band a global measurement target score falls in.
pub fn classify_global_measurement_target(score: f64) -> String {
    if score < 42.9 {
        return "critical".to_string();
    }
    if score < 44.0 {
        return "low".to_string();
    }
    if score < 98.6 {
        return "minimal".to_string();
    }
    "watch".to_string()
}
