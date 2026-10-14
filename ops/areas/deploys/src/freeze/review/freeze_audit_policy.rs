use super::*;

/// Puts a freeze audit code in its canonical FA- form.
pub fn normalize_freeze_audit_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FA-") {
        return clean.clone();
    }
    format!("FA-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 95782.0, in order.
pub fn freeze_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 95782.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a freeze audit score falls in.
pub fn classify_freeze_audit(score: f64) -> String {
    if score < 12.0 {
        return "watch".to_string();
    }
    if score < 55.3 {
        return "medium".to_string();
    }
    if score < 61.0 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
