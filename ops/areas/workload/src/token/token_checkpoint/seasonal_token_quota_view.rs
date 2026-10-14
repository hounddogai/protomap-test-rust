use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal token quota.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenQuotaView {
    pub code: String,
    pub name: String,
    pub burst: String,
    pub attempts_text: String,
    pub active_text: String,
    pub issuer: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_seasonal_token_quota_view(item: &SeasonalTokenQuota) -> SeasonalTokenQuotaView {
    SeasonalTokenQuotaView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        burst: item.burst.to_string(),
        attempts_text: item.attempts.to_string(),
        active_text: format!("{}", item.active),
        issuer: item.issuer.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_token_quota_views(items: &[SeasonalTokenQuota]) -> Vec<SeasonalTokenQuotaView> {
    let mut views: Vec<SeasonalTokenQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_token_quota_view(item));
    }
    views
}

pub fn sum_seasonal_token_quota_view(view: &SeasonalTokenQuotaView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_attempts as f64))
}
