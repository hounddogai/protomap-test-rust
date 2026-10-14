use super::*;

/// How screens and exports show a rolling ttl policy estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingTtlPolicyEstimateView {
    pub code: String,
    pub label: String,
    pub cipher_text: String,
    pub priority_text: String,
    pub active: String,
    pub namespace_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_rolling_ttl_policy_estimate_view(item: &RollingTtlPolicyEstimate) -> RollingTtlPolicyEstimateView {
    RollingTtlPolicyEstimateView {
        code: item.code.clone(),
        label: item.label.clone(),
        cipher_text: item.cipher.clone(),
        priority_text: item.priority.to_string(),
        active: format!("{}", item.active),
        namespace_text: item.namespace.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_ttl_policy_estimate_views(items: &[RollingTtlPolicyEstimate]) -> Vec<RollingTtlPolicyEstimateView> {
    let mut views: Vec<RollingTtlPolicyEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_ttl_policy_estimate_view(item));
    }
    views
}

pub fn sum_rolling_ttl_policy_estimate_view(view: &RollingTtlPolicyEstimateView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_version as f64))
}
