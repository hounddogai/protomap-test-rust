use super::*;

/// Puts a manual measurement exception code in its canonical MME- form.
pub fn normalize_manual_measurement_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MME-") {
        return clean.clone();
    }
    format!("MME-{}", clean.replace(" ", "-"))
}

/// Names the band a manual measurement exception score falls in.
pub fn classify_manual_measurement_exception(score: f64) -> String {
    if score < 20.9 {
        return "low".to_string();
    }
    if score < 65.6 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

/// Counts how many amounts fit in a budget of 45395.0, in order.
pub fn manual_measurement_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45395.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
