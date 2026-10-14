use super::*;

use acme_kit::text;

/// How screens and exports show a pending runbook variance.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookVarianceView {
    pub code_text: String,
    pub title: String,
    pub retries_text: String,
    pub limit: String,
    pub margin: String,
    pub channel: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_pending_runbook_variance_view(item: &PendingRunbookVariance) -> PendingRunbookVarianceView {
    PendingRunbookVarianceView {
        code_text: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 64),
        retries_text: item.retries.to_string(),
        limit: item.limit.to_string(),
        margin: format!("{:.2}", item.margin),
        channel: item.channel.clone(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_runbook_variance_views(items: &[PendingRunbookVariance]) -> Vec<PendingRunbookVarianceView> {
    let mut views: Vec<PendingRunbookVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_runbook_variance_view(item));
    }
    views
}

pub fn sum_pending_runbook_variance_view(view: &PendingRunbookVarianceView) -> Result<f64, String> {
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_limit as f64) + parsed_margin)
}
