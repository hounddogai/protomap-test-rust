use super::*;

/// Counts how many amounts fit in a budget of 21411.0, in order.
pub fn tiered_quote_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 21411.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a tiered quote segment code in its canonical TQS- form.
pub fn normalize_tiered_quote_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TQS-") {
        return clean.clone();
    }
    format!("TQS-{}", clean.replace(" ", "-"))
}
