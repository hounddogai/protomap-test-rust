use super::*;

/// Puts a redaction checkpoint code in its canonical RC- form.
pub fn normalize_redaction_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RC-") {
        return clean.clone();
    }
    format!("RC-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 58469.0, in order.
pub fn redaction_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 58469.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
