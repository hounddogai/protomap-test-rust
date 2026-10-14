use super::*;

/// Puts a dashboard rule code in its canonical DR- form.
pub fn normalize_dashboard_rule_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DR-") {
        return clean.clone();
    }
    format!("DR-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 81281.0, in order.
pub fn dashboard_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 81281.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
