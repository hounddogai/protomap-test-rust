use super::*;

/// Counts how many amounts fit in a budget of 74495.0, in order.
pub fn peering_tier_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 74495.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a peering tier code in its canonical PT- form.
pub fn normalize_peering_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PT-") {
        return clean.clone();
    }
    format!("PT-{}", clean.replace(" ", "-"))
}
