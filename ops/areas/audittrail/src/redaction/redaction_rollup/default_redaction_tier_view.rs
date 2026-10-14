use super::*;

use acme_kit::text;

/// How screens and exports show a default redaction tier.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTierView {
    pub code_text: String,
    pub name_text: String,
    pub source: String,
    pub duration_minutes: String,
    pub starts_at: String,
    pub external_ref: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_default_redaction_tier_view(item: &DefaultRedactionTier) -> DefaultRedactionTierView {
    DefaultRedactionTierView {
        code_text: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        source: item.source.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        starts_at: item.starts_at.to_string(),
        external_ref: item.external_ref.clone(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_redaction_tier_views(items: &[DefaultRedactionTier]) -> Vec<DefaultRedactionTierView> {
    let mut views: Vec<DefaultRedactionTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_redaction_tier_view(item));
    }
    views
}

pub fn sum_default_redaction_tier_view(view: &DefaultRedactionTierView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_starts_at as f64) + (parsed_attempts as f64))
}
