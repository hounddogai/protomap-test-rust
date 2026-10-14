use super::*;

use acme_kit::text;

/// How screens and exports show a capped rollback milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRollbackMilestoneView {
    pub code: String,
    pub name: String,
    pub archived_text: String,
    pub port: String,
    pub amount: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_capped_rollback_milestone_view(item: &CappedRollbackMilestone) -> CappedRollbackMilestoneView {
    CappedRollbackMilestoneView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        archived_text: format!("{}", item.archived),
        port: item.port.to_string(),
        amount: format!("{:.2}", item.amount),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_rollback_milestone_views(items: &[CappedRollbackMilestone]) -> Vec<CappedRollbackMilestoneView> {
    let mut views: Vec<CappedRollbackMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_rollback_milestone_view(item));
    }
    views
}

pub fn sum_capped_rollback_milestone_view(view: &CappedRollbackMilestoneView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_port as f64 + parsed_amount + (parsed_retries as f64))
}
