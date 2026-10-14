use super::*;

/// How screens and exports show a scheduled probe proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeProposalView {
    pub code: String,
    pub label_text: String,
    pub cluster: String,
    pub ends_at: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_scheduled_probe_proposal_view(item: &ScheduledProbeProposal) -> ScheduledProbeProposalView {
    ScheduledProbeProposalView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        cluster: item.cluster.clone(),
        ends_at: item.ends_at.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_probe_proposal_views(items: &[ScheduledProbeProposal]) -> Vec<ScheduledProbeProposalView> {
    let mut views: Vec<ScheduledProbeProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_probe_proposal_view(item));
    }
    views
}

pub fn sum_scheduled_probe_proposal_view(view: &ScheduledProbeProposalView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_ratio)
}
