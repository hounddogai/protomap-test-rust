use super::*;

use acme_kit::text;

/// How screens and exports show a pending runbook estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookEstimateView {
    pub code_text: String,
    pub name_text: String,
    pub maximum: String,
    pub cluster: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_pending_runbook_estimate_view(item: &PendingRunbookEstimate) -> PendingRunbookEstimateView {
    PendingRunbookEstimateView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        maximum: format!("{:.2}", item.maximum),
        cluster: item.cluster.clone(),
        namespace: text::truncate(&item.namespace, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_runbook_estimate_views(items: &[PendingRunbookEstimate]) -> Vec<PendingRunbookEstimateView> {
    let mut views: Vec<PendingRunbookEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_runbook_estimate_view(item));
    }
    views
}

pub fn sum_pending_runbook_estimate_view(view: &PendingRunbookEstimateView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_maximum)
}
