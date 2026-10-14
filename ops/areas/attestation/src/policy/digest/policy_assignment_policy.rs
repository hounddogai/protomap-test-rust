use super::*;

/// Puts a policy assignment code in its canonical PA- form.
pub fn normalize_policy_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PA-") {
        return clean.clone();
    }
    format!("PA-{}", clean.replace(" ", "-"))
}

/// Names the band a policy assignment score falls in.
pub fn classify_policy_assignment(score: f64) -> String {
    if score < 18.2 {
        return "minimal".to_string();
    }
    if score < 34.9 {
        return "critical".to_string();
    }
    "elevated".to_string()
}
