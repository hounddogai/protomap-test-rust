use super::*;

use acme_kit::text;

/// How screens and exports show a flow log agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogAgreementView {
    pub code: String,
    pub label: String,
    pub ceiling_text: String,
    pub namespace: String,
    pub key_id: String,
    pub segment: String,
    pub limit: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_flow_log_agreement_view(item: &FlowLogAgreement) -> FlowLogAgreementView {
    FlowLogAgreementView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        ceiling_text: format!("{:.2}", item.ceiling),
        namespace: text::truncate(&item.namespace, 120),
        key_id: text::truncate(&item.key_id, 120),
        segment: item.segment.clone(),
        limit: item.limit.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_agreement_views(items: &[FlowLogAgreement]) -> Vec<FlowLogAgreementView> {
    let mut views: Vec<FlowLogAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_agreement_view(item));
    }
    views
}

pub fn sum_flow_log_agreement_view(view: &FlowLogAgreementView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ceiling + (parsed_limit as f64) + parsed_ratio)
}
