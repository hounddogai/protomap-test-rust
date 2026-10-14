use super::*;

use acme_kit::text;

/// How screens and exports show a sink rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkRollupView {
    pub code_text: String,
    pub title: String,
    pub archived_text: String,
    pub duration_minutes: String,
    pub phone_text: String,
    pub port: String,
    pub scope_text: String,
    pub timezone: String,
    pub created_at: String,
    pub weight: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_sink_rollup_view(item: &SinkRollup) -> SinkRollupView {
    SinkRollupView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        archived_text: format!("{}", item.archived),
        duration_minutes: item.duration_minutes.to_string(),
        phone_text: item.phone.clone(),
        port: item.port.to_string(),
        scope_text: item.scope.clone(),
        timezone: text::truncate(&item.timezone, 64),
        created_at: item.created_at.to_string(),
        weight: format!("{:.2}", item.weight),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_rollup_views(items: &[SinkRollup]) -> Vec<SinkRollupView> {
    let mut views: Vec<SinkRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_rollup_view(item));
    }
    views
}

pub fn sum_sink_rollup_view(view: &SinkRollupView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_port as f64) + (parsed_created_at as f64))
}
