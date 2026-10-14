use super::*;

/// How screens and exports show a fallback verifier target.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierTargetView {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub retries: String,
    pub cluster: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_fallback_verifier_target_view(item: &FallbackVerifierTarget) -> FallbackVerifierTargetView {
    FallbackVerifierTargetView {
        code: item.code.clone(),
        name: item.name.clone(),
        timezone: item.timezone.clone(),
        retries: item.retries.to_string(),
        cluster: item.cluster.clone(),
        label: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_verifier_target_views(items: &[FallbackVerifierTarget]) -> Vec<FallbackVerifierTargetView> {
    let mut views: Vec<FallbackVerifierTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_verifier_target_view(item));
    }
    views
}

pub fn sum_fallback_verifier_target_view(view: &FallbackVerifierTargetView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_retries as f64)
}
