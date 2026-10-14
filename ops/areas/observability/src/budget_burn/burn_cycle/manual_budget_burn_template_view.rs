use super::*;

use acme_kit::text;

/// How screens and exports show a manual budget burn template.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnTemplateView {
    pub code: String,
    pub title_text: String,
    pub pinned: String,
    pub cluster: String,
    pub archived_text: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_manual_budget_burn_template_view(item: &ManualBudgetBurnTemplate) -> ManualBudgetBurnTemplateView {
    ManualBudgetBurnTemplateView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        pinned: format!("{}", item.pinned),
        cluster: text::truncate(&item.cluster, 120),
        archived_text: format!("{}", item.archived),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_budget_burn_template_views(items: &[ManualBudgetBurnTemplate]) -> Vec<ManualBudgetBurnTemplateView> {
    let mut views: Vec<ManualBudgetBurnTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_budget_burn_template_view(item));
    }
    views
}

pub fn sum_manual_budget_burn_template_view(view: &ManualBudgetBurnTemplateView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
