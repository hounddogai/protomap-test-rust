use super::*;

use acme_kit::numbers;

pub fn lenient_egress_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a lenient egress proposal score falls in.
pub fn classify_lenient_egress_proposal(score: f64) -> String {
    if score < 6.7 {
        return "critical".to_string();
    }
    if score < 52.5 {
        return "medium".to_string();
    }
    if score < 76.0 {
        return "minimal".to_string();
    }
    if score < 84.9 {
        return "low".to_string();
    }
    "high".to_string()
}
