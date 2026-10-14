use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled timeline checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineCheckpointView {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub quota_units: String,
    pub active: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_scheduled_timeline_checkpoint_view(item: &ScheduledTimelineCheckpoint) -> ScheduledTimelineCheckpointView {
    ScheduledTimelineCheckpointView {
        code: item.code.clone(),
        title: item.title.clone(),
        channel: text::truncate(&item.channel, 64),
        quota_units: item.quota_units.to_string(),
        active: format!("{}", item.active),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_timeline_checkpoint_views(
    items: &[ScheduledTimelineCheckpoint],
) -> Vec<ScheduledTimelineCheckpointView> {
    let mut views: Vec<ScheduledTimelineCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_timeline_checkpoint_view(item));
    }
    views
}

pub fn sum_scheduled_timeline_checkpoint_view(view: &ScheduledTimelineCheckpointView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_limit as f64))
}
