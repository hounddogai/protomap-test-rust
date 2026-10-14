use super::*;

/// Counts how many amounts fit in a budget of 72362.0, in order.
pub fn lenient_redaction_milestone_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72362.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a lenient redaction milestone code in its canonical LRM- form.
pub fn normalize_lenient_redaction_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRM-") {
        return clean.clone();
    }
    format!("LRM-{}", clean.replace(" ", "-"))
}
