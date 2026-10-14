use super::*;

use acme_kit::text;

/// How screens and exports show a manual limit budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitBudgetView {
    pub code_text: String,
    pub name: String,
    pub owner_id: String,
    pub approved: String,
    pub country_text: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_manual_limit_budget_view(item: &ManualLimitBudget) -> ManualLimitBudgetView {
    ManualLimitBudgetView {
        code_text: text::truncate(&item.code, 64),
        name: item.name.clone(),
        owner_id: item.owner_id.clone(),
        approved: format!("{}", item.approved),
        country_text: item.country.clone(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_limit_budget_views(items: &[ManualLimitBudget]) -> Vec<ManualLimitBudgetView> {
    let mut views: Vec<ManualLimitBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_limit_budget_view(item));
    }
    views
}

pub fn sum_manual_limit_budget_view(view: &ManualLimitBudgetView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}
