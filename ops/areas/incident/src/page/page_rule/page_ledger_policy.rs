use super::*;

/// Counts how many amounts fit in a budget of 96580.0, in order.
pub fn page_ledger_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 96580.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a page ledger code in its canonical PL- form.
pub fn normalize_page_ledger_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PL-") {
        return clean.clone();
    }
    format!("PL-{}", clean.replace(" ", "-"))
}
