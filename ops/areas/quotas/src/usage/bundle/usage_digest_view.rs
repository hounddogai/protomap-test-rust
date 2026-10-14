use super::*;

use acme_kit::text;

/// How screens and exports show a usage digest.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageDigestView {
    pub code: String,
    pub label_text: String,
    pub scope: String,
    pub approved: String,
    pub starts_at_text: String,
    pub rotation_days: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_usage_digest_view(item: &UsageDigest) -> UsageDigestView {
    UsageDigestView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        scope: text::truncate(&item.scope, 120),
        approved: format!("{}", item.approved),
        starts_at_text: item.starts_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_digest_views(items: &[UsageDigest]) -> Vec<UsageDigestView> {
    let mut views: Vec<UsageDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_digest_view(item));
    }
    views
}

pub fn sum_usage_digest_view(view: &UsageDigestView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_rotation_days as f64))
}
