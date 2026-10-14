use super::*;

/// How screens and exports show a batched ttl policy manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyManifestView {
    pub code: String,
    pub name: String,
    pub window_days: String,
    pub cost: String,
    pub created_at: String,
    pub key_id_text: String,
    pub locale_text: String,
    pub cluster_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_batched_ttl_policy_manifest_view(item: &BatchedTtlPolicyManifest) -> BatchedTtlPolicyManifestView {
    BatchedTtlPolicyManifestView {
        code: item.code.clone(),
        name: item.name.clone(),
        window_days: item.window_days.to_string(),
        cost: format!("{:.2}", item.cost),
        created_at: item.created_at.to_string(),
        key_id_text: item.key_id.clone(),
        locale_text: item.locale.clone(),
        cluster_text: item.cluster.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_ttl_policy_manifest_views(items: &[BatchedTtlPolicyManifest]) -> Vec<BatchedTtlPolicyManifestView> {
    let mut views: Vec<BatchedTtlPolicyManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_ttl_policy_manifest_view(item));
    }
    views
}

pub fn sum_batched_ttl_policy_manifest_view(view: &BatchedTtlPolicyManifestView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_cost + (parsed_created_at as f64))
}
