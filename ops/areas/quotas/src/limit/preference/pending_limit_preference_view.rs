use super::*;

/// How screens and exports show a pending limit preference.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPreferenceView {
    pub code_text: String,
    pub label: String,
    pub discount: String,
    pub rotation_days: String,
    pub weight: String,
    pub quantity_text: String,
    pub score_text: String,
    pub window_days: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_pending_limit_preference_view(item: &PendingLimitPreference) -> PendingLimitPreferenceView {
    PendingLimitPreferenceView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        rotation_days: item.rotation_days.to_string(),
        weight: format!("{:.2}", item.weight),
        quantity_text: item.quantity.to_string(),
        score_text: format!("{:.2}", item.score),
        window_days: item.window_days.to_string(),
        checksum_text: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_limit_preference_views(items: &[PendingLimitPreference]) -> Vec<PendingLimitPreferenceView> {
    let mut views: Vec<PendingLimitPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_limit_preference_view(item));
    }
    views
}

pub fn sum_pending_limit_preference_view(view: &PendingLimitPreferenceView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_discount + (parsed_rotation_days as f64) + parsed_weight)
}
