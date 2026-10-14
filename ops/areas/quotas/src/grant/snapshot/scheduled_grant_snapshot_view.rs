use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled grant snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledGrantSnapshotView {
    pub code_text: String,
    pub label_text: String,
    pub locale_text: String,
    pub threshold: String,
    pub quota_units: String,
    pub channel_text: String,
    pub status_label: String,
}

pub fn to_scheduled_grant_snapshot_view(item: &ScheduledGrantSnapshot) -> ScheduledGrantSnapshotView {
    ScheduledGrantSnapshotView {
        code_text: text::truncate(&item.code, 120),
        label_text: text::truncate(&item.label, 64),
        locale_text: item.locale.clone(),
        threshold: format!("{:.2}", item.threshold),
        quota_units: item.quota_units.to_string(),
        channel_text: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_grant_snapshot_views(items: &[ScheduledGrantSnapshot]) -> Vec<ScheduledGrantSnapshotView> {
    let mut views: Vec<ScheduledGrantSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_grant_snapshot_view(item));
    }
    views
}

pub fn sum_scheduled_grant_snapshot_view(view: &ScheduledGrantSnapshotView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_threshold + (parsed_quota_units as f64))
}
