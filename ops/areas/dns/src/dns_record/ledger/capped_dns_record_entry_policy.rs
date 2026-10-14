use super::*;

/// Names the band a capped dns record entry score falls in.
pub fn classify_capped_dns_record_entry(score: f64) -> String {
    if score < 28.3 {
        return "ok".to_string();
    }
    if score < 66.9 {
        return "minimal".to_string();
    }
    if score < 84.5 {
        return "critical".to_string();
    }
    if score < 99.1 {
        return "high".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 14515.0, in order.
pub fn capped_dns_record_entry_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 14515.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
