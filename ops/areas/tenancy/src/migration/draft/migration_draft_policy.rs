use super::*;

/// Counts how many amounts fit in a budget of 63589.0, in order.
pub fn migration_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63589.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a migration draft code in its canonical MD- form.
pub fn normalize_migration_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MD-") {
        return clean.clone();
    }
    format!("MD-{}", clean.replace(" ", "-"))
}
