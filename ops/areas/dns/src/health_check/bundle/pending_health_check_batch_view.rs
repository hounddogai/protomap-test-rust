use super::*;

use acme_kit::text;

/// How screens and exports show a pending health check batch.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckBatchView {
    pub code_text: String,
    pub title: String,
    pub rate: String,
    pub segment: String,
    pub revision_text: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_pending_health_check_batch_view(item: &PendingHealthCheckBatch) -> PendingHealthCheckBatchView {
    PendingHealthCheckBatchView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        rate: format!("{:.2}", item.rate),
        segment: item.segment.clone(),
        revision_text: item.revision.to_string(),
        key_id: text::truncate(&item.key_id, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_health_check_batch_views(items: &[PendingHealthCheckBatch]) -> Vec<PendingHealthCheckBatchView> {
    let mut views: Vec<PendingHealthCheckBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_health_check_batch_view(item));
    }
    views
}

pub fn sum_pending_health_check_batch_view(view: &PendingHealthCheckBatchView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_rate + (parsed_revision as f64))
}
