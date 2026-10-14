use super::*;

use acme_kit::text;

/// How screens and exports show a primary flow log record.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFlowLogRecordView {
    pub code: String,
    pub label_text: String,
    pub audience: String,
    pub amount: String,
    pub expires_at: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_primary_flow_log_record_view(item: &PrimaryFlowLogRecord) -> PrimaryFlowLogRecordView {
    PrimaryFlowLogRecordView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        audience: item.audience.clone(),
        amount: format!("{:.2}", item.amount),
        expires_at: item.expires_at.to_string(),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_flow_log_record_views(items: &[PrimaryFlowLogRecord]) -> Vec<PrimaryFlowLogRecordView> {
    let mut views: Vec<PrimaryFlowLogRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_flow_log_record_view(item));
    }
    views
}

pub fn sum_primary_flow_log_record_view(view: &PrimaryFlowLogRecordView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_amount + (parsed_expires_at as f64))
}
