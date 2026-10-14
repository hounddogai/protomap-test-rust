use super::*;

/// Counts how many amounts fit in a budget of 25891.0, in order.
pub fn tenant_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 25891.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a tenant rollup code in its canonical TR- form.
pub fn normalize_tenant_rollup_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TR-") {
        return clean.clone();
    }
    format!("TR-{}", clean.replace(" ", "-"))
}
