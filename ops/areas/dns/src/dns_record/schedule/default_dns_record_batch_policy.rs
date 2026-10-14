use super::*;

/// Names the band a default dns record batch score falls in.
pub fn classify_default_dns_record_batch(score: f64) -> String {
    if score < 18.1 {
        return "critical".to_string();
    }
    if score < 54.9 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Puts a default dns record batch code in its canonical DDRB- form.
pub fn normalize_default_dns_record_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DDRB-") {
        return clean.clone();
    }
    format!("DDRB-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 77219.0, in order.
pub fn default_dns_record_batch_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 77219.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn default_dns_record_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}
