use super::*;

/// Puts a lenient page exception code in its canonical LPE- form.
pub fn normalize_lenient_page_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LPE-") {
        return clean.clone();
    }
    format!("LPE-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 13884.0, in order.
pub fn lenient_page_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 13884.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn lenient_page_exception_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 60
}
