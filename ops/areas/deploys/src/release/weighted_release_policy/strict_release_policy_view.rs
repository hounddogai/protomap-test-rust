use super::*;

/// How screens and exports show a strict release policy.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictReleasePolicyView {
    pub code: String,
    pub name: String,
    pub capacity: String,
    pub ttl_seconds_text: String,
    pub category_text: String,
    pub active: String,
    pub rate: String,
    pub fingerprint_text: String,
    pub key_id: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_strict_release_policy_view(item: &StrictReleasePolicy) -> StrictReleasePolicyView {
    StrictReleasePolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        capacity: item.capacity.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        category_text: item.category.clone(),
        active: format!("{}", item.active),
        rate: format!("{:.2}", item.rate),
        fingerprint_text: item.fingerprint.clone(),
        key_id: item.key_id.clone(),
        cluster_text: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_release_policy_views(items: &[StrictReleasePolicy]) -> Vec<StrictReleasePolicyView> {
    let mut views: Vec<StrictReleasePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_release_policy_view(item));
    }
    views
}

pub fn sum_strict_release_policy_view(view: &StrictReleasePolicyView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_ttl_seconds as f64) + parsed_rate)
}
