use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMilestoneView {
    pub code: String,
    pub name: String,
    pub maximum_text: String,
    pub audience: String,
    pub score: String,
    pub version_text: String,
    pub priority: String,
    pub rotation_days: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_budget_burn_milestone_view(item: &BudgetBurnMilestone) -> BudgetBurnMilestoneView {
    BudgetBurnMilestoneView {
        code: item.code.clone(),
        name: item.name.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        audience: text::truncate(&item.audience, 64),
        score: format!("{:.2}", item.score),
        version_text: item.version.to_string(),
        priority: item.priority.to_string(),
        rotation_days: item.rotation_days.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_milestone_views(items: &[BudgetBurnMilestone]) -> Vec<BudgetBurnMilestoneView> {
    let mut views: Vec<BudgetBurnMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_milestone_view(item));
    }
    views
}

pub fn sum_budget_burn_milestone_view(view: &BudgetBurnMilestoneView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_maximum + parsed_score + (parsed_version as f64))
}
