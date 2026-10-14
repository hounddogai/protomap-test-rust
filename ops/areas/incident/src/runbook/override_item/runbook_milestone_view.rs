use super::*;

use acme_kit::text;

/// How screens and exports show a runbook milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMilestoneView {
    pub code: String,
    pub name_text: String,
    pub ends_at: String,
    pub audience: String,
    pub discount_text: String,
    pub created_at: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_runbook_milestone_view(item: &RunbookMilestone) -> RunbookMilestoneView {
    RunbookMilestoneView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        ends_at: item.ends_at.to_string(),
        audience: item.audience.clone(),
        discount_text: format!("{:.2}", item.discount),
        created_at: item.created_at.to_string(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_milestone_views(items: &[RunbookMilestone]) -> Vec<RunbookMilestoneView> {
    let mut views: Vec<RunbookMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_milestone_view(item));
    }
    views
}

pub fn sum_runbook_milestone_view(view: &RunbookMilestoneView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_discount + (parsed_created_at as f64))
}
