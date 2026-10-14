use super::*;

/// How screens and exports show a manual verifier preference.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierPreferenceView {
    pub code_text: String,
    pub title: String,
    pub pinned_text: String,
    pub fingerprint: String,
    pub quantity: String,
    pub maximum: String,
    pub description: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_manual_verifier_preference_view(item: &ManualVerifierPreference) -> ManualVerifierPreferenceView {
    ManualVerifierPreferenceView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        pinned_text: format!("{}", item.pinned),
        fingerprint: item.fingerprint.clone(),
        quantity: item.quantity.to_string(),
        maximum: format!("{:.2}", item.maximum),
        description: item.description.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_verifier_preference_views(items: &[ManualVerifierPreference]) -> Vec<ManualVerifierPreferenceView> {
    let mut views: Vec<ManualVerifierPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_verifier_preference_view(item));
    }
    views
}

pub fn sum_manual_verifier_preference_view(view: &ManualVerifierPreferenceView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_maximum + parsed_score)
}
