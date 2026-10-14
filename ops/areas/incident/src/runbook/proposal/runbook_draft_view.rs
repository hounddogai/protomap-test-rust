use super::*;

use acme_kit::text;

/// How screens and exports show a runbook draft.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookDraftView {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub retries_text: String,
    pub archived: String,
    pub rotation_days: String,
    pub cipher_text: String,
    pub quantity: String,
    pub burst: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_runbook_draft_view(item: &RunbookDraft) -> RunbookDraftView {
    RunbookDraftView {
        code: item.code.clone(),
        label: item.label.clone(),
        summary: text::truncate(&item.summary, 40),
        retries_text: item.retries.to_string(),
        archived: format!("{}", item.archived),
        rotation_days: item.rotation_days.to_string(),
        cipher_text: item.cipher.clone(),
        quantity: item.quantity.to_string(),
        burst: item.burst.to_string(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_draft_views(items: &[RunbookDraft]) -> Vec<RunbookDraftView> {
    let mut views: Vec<RunbookDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_draft_view(item));
    }
    views
}

pub fn sum_runbook_draft_view(view: &RunbookDraftView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_rotation_days as f64) + (parsed_quantity as f64))
}
