use super::*;

use acme_kit::text;

/// How screens and exports show a evidence plan.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePlanView {
    pub code: String,
    pub label: String,
    pub source: String,
    pub title: String,
    pub port_text: String,
    pub phone: String,
    pub pinned_text: String,
    pub channel_text: String,
    pub status_label: String,
}

pub fn to_evidence_plan_view(item: &EvidencePlan) -> EvidencePlanView {
    EvidencePlanView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        source: item.source.clone(),
        title: item.title.clone(),
        port_text: item.port.to_string(),
        phone: item.phone.clone(),
        pinned_text: format!("{}", item.pinned),
        channel_text: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_plan_views(items: &[EvidencePlan]) -> Vec<EvidencePlanView> {
    let mut views: Vec<EvidencePlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_plan_view(item));
    }
    views
}

pub fn sum_evidence_plan_view(view: &EvidencePlanView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}
