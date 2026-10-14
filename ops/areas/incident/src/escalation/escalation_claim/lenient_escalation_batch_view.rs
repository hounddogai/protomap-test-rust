use super::*;

use acme_kit::text;

/// How screens and exports show a lenient escalation batch.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEscalationBatchView {
    pub code_text: String,
    pub label_text: String,
    pub host: String,
    pub issuer: String,
    pub notes: String,
    pub principal_text: String,
    pub rate: String,
    pub archived_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_lenient_escalation_batch_view(item: &LenientEscalationBatch) -> LenientEscalationBatchView {
    LenientEscalationBatchView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        host: item.host.clone(),
        issuer: item.issuer.clone(),
        notes: item.notes.clone(),
        principal_text: text::truncate(&item.principal, 80),
        rate: format!("{:.2}", item.rate),
        archived_text: format!("{}", item.archived),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_escalation_batch_views(items: &[LenientEscalationBatch]) -> Vec<LenientEscalationBatchView> {
    let mut views: Vec<LenientEscalationBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_escalation_batch_view(item));
    }
    views
}

pub fn sum_lenient_escalation_batch_view(view: &LenientEscalationBatchView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}
