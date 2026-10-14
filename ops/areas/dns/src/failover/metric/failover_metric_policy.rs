use super::*;

/// Counts how many amounts fit in a budget of 7828.0, in order.
pub fn failover_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7828.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a failover metric code in its canonical FM- form.
pub fn normalize_failover_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FM-") {
        return clean.clone();
    }
    format!("FM-{}", clean.replace(" ", "-"))
}
