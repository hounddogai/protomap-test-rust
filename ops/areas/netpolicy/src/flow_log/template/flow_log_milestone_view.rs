use super::*;

use acme_kit::text;

/// How screens and exports show a flow log milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogMilestoneView {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub locale: String,
    pub issuer: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_flow_log_milestone_view(item: &FlowLogMilestone) -> FlowLogMilestoneView {
    FlowLogMilestoneView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        fingerprint: item.fingerprint.clone(),
        locale: item.locale.clone(),
        issuer: item.issuer.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_milestone_views(items: &[FlowLogMilestone]) -> Vec<FlowLogMilestoneView> {
    let mut views: Vec<FlowLogMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_milestone_view(item));
    }
    views
}

pub fn sum_flow_log_milestone_view(view: &FlowLogMilestoneView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}
