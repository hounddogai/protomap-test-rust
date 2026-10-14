use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled zone override.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneOverrideView {
    pub code: String,
    pub name_text: String,
    pub sequence: String,
    pub rotation_days: String,
    pub host: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_scheduled_zone_override_view(item: &ScheduledZoneOverride) -> ScheduledZoneOverrideView {
    ScheduledZoneOverrideView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        sequence: item.sequence.to_string(),
        rotation_days: item.rotation_days.to_string(),
        host: item.host.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_zone_override_views(items: &[ScheduledZoneOverride]) -> Vec<ScheduledZoneOverrideView> {
    let mut views: Vec<ScheduledZoneOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_zone_override_view(item));
    }
    views
}

pub fn sum_scheduled_zone_override_view(view: &ScheduledZoneOverrideView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_rotation_days as f64) + (parsed_revision as f64))
}
