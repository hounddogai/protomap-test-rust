use super::*;

use acme_kit::text;

/// How screens and exports show a legacy quote checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQuoteCheckpointView {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub principal: String,
    pub region: String,
    pub priority_text: String,
    pub country_text: String,
    pub updated_at: String,
    pub timezone: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_legacy_quote_checkpoint_view(item: &LegacyQuoteCheckpoint) -> LegacyQuoteCheckpointView {
    LegacyQuoteCheckpointView {
        code: item.code.clone(),
        title: item.title.clone(),
        key_id: item.key_id.clone(),
        principal: text::truncate(&item.principal, 120),
        region: item.region.clone(),
        priority_text: item.priority.to_string(),
        country_text: text::truncate(&item.country, 40),
        updated_at: item.updated_at.to_string(),
        timezone: item.timezone.clone(),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_quote_checkpoint_views(items: &[LegacyQuoteCheckpoint]) -> Vec<LegacyQuoteCheckpointView> {
    let mut views: Vec<LegacyQuoteCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_quote_checkpoint_view(item));
    }
    views
}

pub fn sum_legacy_quote_checkpoint_view(view: &LegacyQuoteCheckpointView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_updated_at as f64) + (parsed_grace_days as f64))
}
