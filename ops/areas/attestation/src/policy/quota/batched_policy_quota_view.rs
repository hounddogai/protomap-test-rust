use super::*;

use acme_kit::text;

/// How screens and exports show a batched policy quota.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyQuotaView {
    pub code: String,
    pub label_text: String,
    pub cluster: String,
    pub audience: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_batched_policy_quota_view(item: &BatchedPolicyQuota) -> BatchedPolicyQuotaView {
    BatchedPolicyQuotaView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        cluster: text::truncate(&item.cluster, 64),
        audience: item.audience.clone(),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_policy_quota_views(items: &[BatchedPolicyQuota]) -> Vec<BatchedPolicyQuotaView> {
    let mut views: Vec<BatchedPolicyQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_policy_quota_view(item));
    }
    views
}

pub fn sum_batched_policy_quota_view(view: &BatchedPolicyQuotaView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}
