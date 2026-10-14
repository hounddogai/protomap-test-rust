use super::*;

/// Counts how many amounts fit in a budget of 72273.0, in order.
pub fn health_check_score_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72273.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a health check score code in its canonical HCS- form.
pub fn normalize_health_check_score_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCS-") {
        return clean.clone();
    }
    format!("HCS-{}", clean.replace(" ", "-"))
}
