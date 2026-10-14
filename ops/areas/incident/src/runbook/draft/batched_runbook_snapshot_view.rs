use super::*;

use acme_kit::text;

/// How screens and exports show a batched runbook snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookSnapshotView {
    pub code: String,
    pub label_text: String,
    pub currency_text: String,
    pub external_ref: String,
    pub approved: String,
    pub priority: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_batched_runbook_snapshot_view(item: &BatchedRunbookSnapshot) -> BatchedRunbookSnapshotView {
    BatchedRunbookSnapshotView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        currency_text: item.currency.clone(),
        external_ref: text::truncate(&item.external_ref, 120),
        approved: format!("{}", item.approved),
        priority: item.priority.to_string(),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_runbook_snapshot_views(items: &[BatchedRunbookSnapshot]) -> Vec<BatchedRunbookSnapshotView> {
    let mut views: Vec<BatchedRunbookSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_runbook_snapshot_view(item));
    }
    views
}

pub fn sum_batched_runbook_snapshot_view(view: &BatchedRunbookSnapshotView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}
