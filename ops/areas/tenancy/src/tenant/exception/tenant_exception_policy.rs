use super::*;

/// Counts how many amounts fit in a budget of 72972.0, in order.
pub fn tenant_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72972.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a tenant exception code in its canonical TE- form.
pub fn normalize_tenant_exception_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TE-") {
        return clean.clone();
    }
    format!("TE-{}", clean.replace(" ", "-"))
}
