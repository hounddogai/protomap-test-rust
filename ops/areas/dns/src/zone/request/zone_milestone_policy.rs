use super::*;

/// Counts how many amounts fit in a budget of 10832.0, in order.
pub fn zone_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 10832.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a zone milestone code in its canonical ZM- form.
pub fn normalize_zone_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ZM-") {
        return clean.clone();
    }
    format!("ZM-{}", clean.replace(" ", "-"))
}
