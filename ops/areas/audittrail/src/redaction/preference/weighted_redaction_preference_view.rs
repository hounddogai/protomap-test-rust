use super::*;

/// How screens and exports show a weighted redaction preference.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRedactionPreferenceView {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub archived_text: String,
    pub fingerprint_text: String,
    pub threshold_text: String,
    pub expires_at: String,
    pub created_at_text: String,
    pub score_text: String,
    pub attempts: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_weighted_redaction_preference_view(item: &WeightedRedactionPreference) -> WeightedRedactionPreferenceView {
    WeightedRedactionPreferenceView {
        code: item.code.clone(),
        title: item.title.clone(),
        summary: item.summary.clone(),
        archived_text: format!("{}", item.archived),
        fingerprint_text: item.fingerprint.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        expires_at: item.expires_at.to_string(),
        created_at_text: item.created_at.to_string(),
        score_text: format!("{:.2}", item.score),
        attempts: item.attempts.to_string(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_redaction_preference_views(
    items: &[WeightedRedactionPreference],
) -> Vec<WeightedRedactionPreferenceView> {
    let mut views: Vec<WeightedRedactionPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_redaction_preference_view(item));
    }
    views
}

pub fn sum_weighted_redaction_preference_view(view: &WeightedRedactionPreferenceView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_threshold + (parsed_expires_at as f64) + (parsed_created_at as f64))
}
