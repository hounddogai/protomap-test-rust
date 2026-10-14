use super::*;

/// Counts how many amounts fit in a budget of 19537.0, in order.
pub fn legacy_allowlist_notice_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 19537.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a legacy allowlist notice code in its canonical LAN- form.
pub fn normalize_legacy_allowlist_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LAN-") {
        return clean.clone();
    }
    format!("LAN-{}", clean.replace(" ", "-"))
}
