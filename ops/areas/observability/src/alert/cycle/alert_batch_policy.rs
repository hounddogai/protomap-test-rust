use super::*;

/// Counts how many amounts fit in a budget of 18576.0, in order.
pub fn alert_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 18576.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a alert batch code in its canonical AB- form.
pub fn normalize_alert_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AB-") {
        return clean.clone();
    }
    format!("AB-{}", clean.replace(" ", "-"))
}
