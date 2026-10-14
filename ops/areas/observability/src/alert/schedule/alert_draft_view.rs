use super::*;

use acme_kit::text;

/// How screens and exports show a alert draft.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertDraftView {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub reviewer: String,
    pub version: String,
    pub channel: String,
    pub window_days_text: String,
    pub archived: String,
    pub cluster: String,
    pub limit_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_alert_draft_view(item: &AlertDraft) -> AlertDraftView {
    AlertDraftView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        unit: item.unit.clone(),
        reviewer: item.reviewer.clone(),
        version: item.version.to_string(),
        channel: item.channel.clone(),
        window_days_text: item.window_days.to_string(),
        archived: format!("{}", item.archived),
        cluster: item.cluster.clone(),
        limit_text: item.limit.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_draft_views(items: &[AlertDraft]) -> Vec<AlertDraftView> {
    let mut views: Vec<AlertDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_draft_view(item));
    }
    views
}

pub fn sum_alert_draft_view(view: &AlertDraftView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_version as f64 + (parsed_window_days as f64) + (parsed_limit as f64))
}
