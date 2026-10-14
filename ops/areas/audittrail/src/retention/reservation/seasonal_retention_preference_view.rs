use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal retention preference.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionPreferenceView {
    pub code: String,
    pub name_text: String,
    pub maximum: String,
    pub owner_id_text: String,
    pub scope: String,
    pub host: String,
    pub retries: String,
    pub grace_days_text: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_seasonal_retention_preference_view(item: &SeasonalRetentionPreference) -> SeasonalRetentionPreferenceView {
    SeasonalRetentionPreferenceView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        maximum: format!("{:.2}", item.maximum),
        owner_id_text: item.owner_id.clone(),
        scope: text::truncate(&item.scope, 80),
        host: text::truncate(&item.host, 64),
        retries: item.retries.to_string(),
        grace_days_text: item.grace_days.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_retention_preference_views(
    items: &[SeasonalRetentionPreference],
) -> Vec<SeasonalRetentionPreferenceView> {
    let mut views: Vec<SeasonalRetentionPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_retention_preference_view(item));
    }
    views
}

pub fn sum_seasonal_retention_preference_view(view: &SeasonalRetentionPreferenceView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_maximum + (parsed_retries as f64) + (parsed_grace_days as f64))
}
