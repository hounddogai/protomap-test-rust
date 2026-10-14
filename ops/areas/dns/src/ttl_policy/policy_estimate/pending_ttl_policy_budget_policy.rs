use super::*;

/// Names the band a pending ttl policy budget score falls in.
pub fn classify_pending_ttl_policy_budget(score: f64) -> String {
    if score < 18.6 {
        return "minimal".to_string();
    }
    if score < 48.5 {
        return "medium".to_string();
    }
    if score < 54.1 {
        return "high".to_string();
    }
    if score < 75.9 {
        return "low".to_string();
    }
    "watch".to_string()
}

/// Puts a pending ttl policy budget code in its canonical PTPB- form.
pub fn normalize_pending_ttl_policy_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PTPB-") {
        return clean.clone();
    }
    format!("PTPB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 1258.0, in order.
pub fn pending_ttl_policy_budget_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 1258.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
