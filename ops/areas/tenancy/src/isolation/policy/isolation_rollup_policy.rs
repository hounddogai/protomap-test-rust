use super::*;

/// Counts how many amounts fit in a budget of 32868.0, in order.
pub fn isolation_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 32868.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a isolation rollup code in its canonical IR- form.
pub fn normalize_isolation_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("IR-") {
        return clean.clone();
    }
    format!("IR-{}", clean.replace(" ", "-"))
}

/// Names the band a isolation rollup score falls in.
pub fn classify_isolation_rollup(score: f64) -> String {
    if score < 36.6 {
        return "elevated".to_string();
    }
    if score < 57.0 {
        return "critical".to_string();
    }
    if score < 66.9 {
        return "watch".to_string();
    }
    "low".to_string()
}
