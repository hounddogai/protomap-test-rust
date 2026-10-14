use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn plan.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPlanView {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub pinned: String,
    pub phone: String,
    pub threshold: String,
    pub cluster_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_budget_burn_plan_view(item: &BudgetBurnPlan) -> BudgetBurnPlanView {
    BudgetBurnPlanView {
        code: item.code.clone(),
        label: item.label.clone(),
        issuer: item.issuer.clone(),
        pinned: format!("{}", item.pinned),
        phone: text::truncate(&item.phone, 40),
        threshold: format!("{:.2}", item.threshold),
        cluster_text: text::truncate(&item.cluster, 80),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_plan_views(items: &[BudgetBurnPlan]) -> Vec<BudgetBurnPlanView> {
    let mut views: Vec<BudgetBurnPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_plan_view(item));
    }
    views
}

pub fn sum_budget_burn_plan_view(view: &BudgetBurnPlanView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}
