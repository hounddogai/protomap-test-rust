use super::*;

/// Counts how many amounts fit in a budget of 84510.0, in order.
pub fn page_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84510.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a page exception code in its canonical PE- form.
pub fn normalize_page_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PE-") {
        return clean.clone();
    }
    format!("PE-{}", clean.replace(" ", "-"))
}
