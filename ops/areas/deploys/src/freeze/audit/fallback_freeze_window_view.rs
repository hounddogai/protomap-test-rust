use super::*;

use acme_kit::text;

/// How screens and exports show a fallback freeze window.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFreezeWindowView {
    pub code: String,
    pub name: String,
    pub duration_minutes: String,
    pub namespace: String,
    pub key_id_text: String,
    pub rotation_days_text: String,
    pub version: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_fallback_freeze_window_view(item: &FallbackFreezeWindow) -> FallbackFreezeWindowView {
    FallbackFreezeWindowView {
        code: item.code.clone(),
        name: item.name.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        namespace: text::truncate(&item.namespace, 40),
        key_id_text: text::truncate(&item.key_id, 80),
        rotation_days_text: item.rotation_days.to_string(),
        version: item.version.to_string(),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_freeze_window_views(items: &[FallbackFreezeWindow]) -> Vec<FallbackFreezeWindowView> {
    let mut views: Vec<FallbackFreezeWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_freeze_window_view(item));
    }
    views
}

pub fn sum_fallback_freeze_window_view(view: &FallbackFreezeWindowView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_rotation_days as f64) + (parsed_version as f64))
}
