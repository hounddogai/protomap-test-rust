use super::*;

/// How screens and exports show a global limit preference.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalLimitPreferenceView {
    pub code: String,
    pub title_text: String,
    pub quantity_text: String,
    pub archived_text: String,
    pub rotation_days: String,
    pub duration_minutes_text: String,
    pub window_days_text: String,
    pub discount: String,
    pub capacity: String,
    pub rate: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_global_limit_preference_view(item: &GlobalLimitPreference) -> GlobalLimitPreferenceView {
    GlobalLimitPreferenceView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        quantity_text: item.quantity.to_string(),
        archived_text: format!("{}", item.archived),
        rotation_days: item.rotation_days.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        window_days_text: item.window_days.to_string(),
        discount: format!("{:.2}", item.discount),
        capacity: item.capacity.to_string(),
        rate: format!("{:.2}", item.rate),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_limit_preference_views(items: &[GlobalLimitPreference]) -> Vec<GlobalLimitPreferenceView> {
    let mut views: Vec<GlobalLimitPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_limit_preference_view(item));
    }
    views
}

pub fn sum_global_limit_preference_view(view: &GlobalLimitPreferenceView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_rotation_days as f64) + (parsed_duration_minutes as f64))
}
