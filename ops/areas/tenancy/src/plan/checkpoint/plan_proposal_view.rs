use super::*;

use acme_kit::text;

/// How screens and exports show a plan proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanProposalView {
    pub code_text: String,
    pub title: String,
    pub description: String,
    pub threshold: String,
    pub window_days_text: String,
    pub limit_text: String,
    pub burst: String,
    pub unit_text: String,
    pub locale: String,
    pub budget: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_plan_proposal_view(item: &PlanProposal) -> PlanProposalView {
    PlanProposalView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 64),
        description: item.description.clone(),
        threshold: format!("{:.2}", item.threshold),
        window_days_text: item.window_days.to_string(),
        limit_text: item.limit.to_string(),
        burst: item.burst.to_string(),
        unit_text: item.unit.clone(),
        locale: text::truncate(&item.locale, 80),
        budget: format!("{:.2}", item.budget),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_proposal_views(items: &[PlanProposal]) -> Vec<PlanProposalView> {
    let mut views: Vec<PlanProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_proposal_view(item));
    }
    views
}

pub fn sum_plan_proposal_view(view: &PlanProposalView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_threshold + (parsed_window_days as f64) + (parsed_limit as f64))
}
