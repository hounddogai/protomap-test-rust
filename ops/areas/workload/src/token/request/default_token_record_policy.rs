use super::*;

/// Puts a default token record code in its canonical DTR- form.
pub fn normalize_default_token_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DTR-") {
        return clean.clone();
    }
    format!("DTR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 56636.0, in order.
pub fn default_token_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 56636.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
