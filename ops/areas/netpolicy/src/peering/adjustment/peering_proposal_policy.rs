use super::*;

/// Puts a peering proposal code in its canonical PP- form.
pub fn normalize_peering_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PP-") {
        return clean.clone();
    }
    format!("PP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 57012.0, in order.
pub fn peering_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 57012.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
