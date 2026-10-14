use super::*;

use acme_kit::text;

/// How screens and exports show a weighted identity preference.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedIdentityPreferenceView {
    pub code: String,
    pub name: String,
    pub burst_text: String,
    pub principal_text: String,
    pub approved: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_weighted_identity_preference_view(item: &WeightedIdentityPreference) -> WeightedIdentityPreferenceView {
    WeightedIdentityPreferenceView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        burst_text: item.burst.to_string(),
        principal_text: text::truncate(&item.principal, 120),
        approved: format!("{}", item.approved),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_identity_preference_views(
    items: &[WeightedIdentityPreference],
) -> Vec<WeightedIdentityPreferenceView> {
    let mut views: Vec<WeightedIdentityPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_identity_preference_view(item));
    }
    views
}

pub fn sum_weighted_identity_preference_view(view: &WeightedIdentityPreferenceView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}
