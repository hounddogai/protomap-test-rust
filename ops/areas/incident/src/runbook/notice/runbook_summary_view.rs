use super::*;

use acme_kit::text;

/// How screens and exports show a runbook summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSummaryView {
    pub code: String,
    pub name_text: String,
    pub batch_size: String,
    pub unit: String,
    pub updated_at: String,
    pub approved: String,
    pub category: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_runbook_summary_view(item: &RunbookSummary) -> RunbookSummaryView {
    RunbookSummaryView {
        code: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 80),
        batch_size: item.batch_size.to_string(),
        unit: item.unit.clone(),
        updated_at: item.updated_at.to_string(),
        approved: format!("{}", item.approved),
        category: item.category.clone(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_summary_views(items: &[RunbookSummary]) -> Vec<RunbookSummaryView> {
    let mut views: Vec<RunbookSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_summary_view(item));
    }
    views
}

pub fn sum_runbook_summary_view(view: &RunbookSummaryView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_updated_at as f64))
}
