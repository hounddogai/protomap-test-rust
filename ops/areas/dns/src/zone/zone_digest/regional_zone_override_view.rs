use super::*;

/// How screens and exports show a regional zone override.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneOverrideView {
    pub code: String,
    pub label: String,
    pub quota_units: String,
    pub duration_minutes: String,
    pub revision_text: String,
    pub unit: String,
    pub floor_value_text: String,
    pub limit: String,
    pub version: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_regional_zone_override_view(item: &RegionalZoneOverride) -> RegionalZoneOverrideView {
    RegionalZoneOverrideView {
        code: item.code.clone(),
        label: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        revision_text: item.revision.to_string(),
        unit: item.unit.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        limit: item.limit.to_string(),
        version: item.version.to_string(),
        host_text: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_zone_override_views(items: &[RegionalZoneOverride]) -> Vec<RegionalZoneOverrideView> {
    let mut views: Vec<RegionalZoneOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_zone_override_view(item));
    }
    views
}

pub fn sum_regional_zone_override_view(view: &RegionalZoneOverrideView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_duration_minutes as f64) + (parsed_revision as f64))
}
