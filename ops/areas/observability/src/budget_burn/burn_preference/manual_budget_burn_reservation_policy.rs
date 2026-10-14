use super::*;

/// Counts how many amounts fit in a budget of 89852.0, in order.
pub fn manual_budget_burn_reservation_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 89852.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a manual budget burn reservation code in its canonical MBBR- form.
pub fn normalize_manual_budget_burn_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MBBR-") {
        return clean.clone();
    }
    format!("MBBR-{}", clean.replace(" ", "-"))
}

/// Names the band a manual budget burn reservation score falls in.
pub fn classify_manual_budget_burn_reservation(score: f64) -> String {
    if score < 62.5 {
        return "minimal".to_string();
    }
    if score < 93.9 {
        return "watch".to_string();
    }
    "high".to_string()
}
