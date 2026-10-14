use super::*;

/// Puts a budget burn proposal code in its canonical BBP- form.
pub fn normalize_budget_burn_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBP-") {
        return clean.clone();
    }
    format!("BBP-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 38319.0, in order.
pub fn budget_burn_proposal_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 38319.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
