use super::*;

use acme_kit::text;

/// How screens and exports show a tenant budget.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantBudgetView {
    pub code: String,
    pub title: String,
    pub discount: String,
    pub external_ref: String,
    pub timezone: String,
    pub name: String,
    pub capacity: String,
    pub source: String,
    pub starts_at: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_tenant_budget_view(item: &TenantBudget) -> TenantBudgetView {
    TenantBudgetView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 80),
        discount: format!("{:.2}", item.discount),
        external_ref: item.external_ref.clone(),
        timezone: text::truncate(&item.timezone, 64),
        name: item.name.clone(),
        capacity: item.capacity.to_string(),
        source: item.source.clone(),
        starts_at: item.starts_at.to_string(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_budget_views(items: &[TenantBudget]) -> Vec<TenantBudgetView> {
    let mut views: Vec<TenantBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_budget_view(item));
    }
    views
}

pub fn sum_tenant_budget_view(view: &TenantBudgetView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_discount + (parsed_capacity as f64) + (parsed_starts_at as f64))
}
