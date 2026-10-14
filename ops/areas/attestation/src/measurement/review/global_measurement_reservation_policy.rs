use super::*;

/// Counts how many amounts fit in a budget of 64203.0, in order.
pub fn global_measurement_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 64203.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a global measurement reservation code in its canonical GMR- form.
pub fn normalize_global_measurement_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GMR-") {
        return clean.clone();
    }
    format!("GMR-{}", clean.replace(" ", "-"))
}
