use super::*;

/// Counts how many amounts fit in a budget of 97508.0, in order.
pub fn weighted_zone_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 97508.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a weighted zone reservation code in its canonical WZR- form.
pub fn normalize_weighted_zone_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WZR-") {
        return clean.clone();
    }
    format!("WZR-{}", clean.replace(" ", "-"))
}
