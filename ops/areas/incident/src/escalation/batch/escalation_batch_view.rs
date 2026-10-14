use super::*;

use acme_kit::text;

/// How screens and exports show a escalation batch.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBatchView {
    pub code: String,
    pub name: String,
    pub summary_text: String,
    pub segment: String,
    pub weight: String,
    pub timezone: String,
    pub approved: String,
    pub grace_days_text: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_escalation_batch_view(item: &EscalationBatch) -> EscalationBatchView {
    EscalationBatchView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        summary_text: item.summary.clone(),
        segment: item.segment.clone(),
        weight: format!("{:.2}", item.weight),
        timezone: item.timezone.clone(),
        approved: format!("{}", item.approved),
        grace_days_text: item.grace_days.to_string(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_batch_views(items: &[EscalationBatch]) -> Vec<EscalationBatchView> {
    let mut views: Vec<EscalationBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_batch_view(item));
    }
    views
}

pub fn sum_escalation_batch_view(view: &EscalationBatchView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_weight + (parsed_grace_days as f64))
}
