use super::*;

use acme_kit::numbers;

/// Names the band a lenient redaction manifest score falls in.
pub fn classify_lenient_redaction_manifest(score: f64) -> String {
    if score < 24.0 {
        return "critical".to_string();
    }
    if score < 40.7 {
        return "minimal".to_string();
    }
    if score < 85.5 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn lenient_redaction_manifest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
