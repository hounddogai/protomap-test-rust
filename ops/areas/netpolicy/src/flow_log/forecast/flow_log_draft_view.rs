use super::*;

use acme_kit::text;

/// How screens and exports show a flow log draft.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogDraftView {
    pub code: String,
    pub label: String,
    pub tenant_id_text: String,
    pub ends_at: String,
    pub active: String,
    pub region: String,
    pub channel_text: String,
    pub archived_text: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_flow_log_draft_view(item: &FlowLogDraft) -> FlowLogDraftView {
    FlowLogDraftView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        tenant_id_text: text::truncate(&item.tenant_id, 120),
        ends_at: item.ends_at.to_string(),
        active: format!("{}", item.active),
        region: item.region.clone(),
        channel_text: text::truncate(&item.channel, 80),
        archived_text: format!("{}", item.archived),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_draft_views(items: &[FlowLogDraft]) -> Vec<FlowLogDraftView> {
    let mut views: Vec<FlowLogDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_draft_view(item));
    }
    views
}

pub fn sum_flow_log_draft_view(view: &FlowLogDraftView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_window_days as f64))
}
