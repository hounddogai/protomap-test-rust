use super::*;

use acme_kit::text;

/// How screens and exports show a legacy quote quota.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteQuotaView {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub limit_text: String,
    pub title: String,
    pub cluster: String,
    pub host: String,
    pub pinned_text: String,
    pub ends_at: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_legacy_quote_quota_view(item: &LegacyQuoteQuota) -> LegacyQuoteQuotaView {
    LegacyQuoteQuotaView {
        code: item.code.clone(),
        label: item.label.clone(),
        audience: item.audience.clone(),
        limit_text: item.limit.to_string(),
        title: text::truncate(&item.title, 80),
        cluster: item.cluster.clone(),
        host: text::truncate(&item.host, 64),
        pinned_text: format!("{}", item.pinned),
        ends_at: item.ends_at.to_string(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_quote_quota_views(items: &[LegacyQuoteQuota]) -> Vec<LegacyQuoteQuotaView> {
    let mut views: Vec<LegacyQuoteQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_quote_quota_view(item));
    }
    views
}

pub fn sum_legacy_quote_quota_view(view: &LegacyQuoteQuotaView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_ends_at as f64) + (parsed_starts_at as f64))
}
