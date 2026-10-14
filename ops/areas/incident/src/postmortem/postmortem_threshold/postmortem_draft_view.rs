use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem draft.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDraftView {
    pub code: String,
    pub label: String,
    pub capacity: String,
    pub version: String,
    pub retries_text: String,
    pub archived_text: String,
    pub port_text: String,
    pub quota_units: String,
    pub title_text: String,
    pub status_label: String,
}

pub fn to_postmortem_draft_view(item: &PostmortemDraft) -> PostmortemDraftView {
    PostmortemDraftView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        capacity: item.capacity.to_string(),
        version: item.version.to_string(),
        retries_text: item.retries.to_string(),
        archived_text: format!("{}", item.archived),
        port_text: item.port.to_string(),
        quota_units: item.quota_units.to_string(),
        title_text: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_draft_views(items: &[PostmortemDraft]) -> Vec<PostmortemDraftView> {
    let mut views: Vec<PostmortemDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_draft_view(item));
    }
    views
}

pub fn sum_postmortem_draft_view(view: &PostmortemDraftView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_version as f64) + (parsed_retries as f64))
}
