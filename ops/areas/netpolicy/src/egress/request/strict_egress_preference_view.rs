use super::*;

use acme_kit::text;

/// How screens and exports show a strict egress preference.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEgressPreferenceView {
    pub code: String,
    pub label_text: String,
    pub rotation_days_text: String,
    pub country_text: String,
    pub segment: String,
    pub key_id: String,
    pub maximum: String,
    pub owner_id_text: String,
    pub attempts: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_strict_egress_preference_view(item: &StrictEgressPreference) -> StrictEgressPreferenceView {
    StrictEgressPreferenceView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        country_text: item.country.clone(),
        segment: text::truncate(&item.segment, 80),
        key_id: item.key_id.clone(),
        maximum: format!("{:.2}", item.maximum),
        owner_id_text: item.owner_id.clone(),
        attempts: item.attempts.to_string(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_egress_preference_views(items: &[StrictEgressPreference]) -> Vec<StrictEgressPreferenceView> {
    let mut views: Vec<StrictEgressPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_egress_preference_view(item));
    }
    views
}

pub fn sum_strict_egress_preference_view(view: &StrictEgressPreferenceView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_maximum + (parsed_attempts as f64))
}
