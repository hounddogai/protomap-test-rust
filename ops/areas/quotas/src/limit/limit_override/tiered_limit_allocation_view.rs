use super::*;

/// How screens and exports show a tiered limit allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredLimitAllocationView {
    pub code: String,
    pub name: String,
    pub amount_text: String,
    pub ends_at_text: String,
    pub maximum: String,
    pub revision: String,
    pub approved_text: String,
    pub source: String,
    pub category: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_tiered_limit_allocation_view(item: &TieredLimitAllocation) -> TieredLimitAllocationView {
    TieredLimitAllocationView {
        code: item.code.clone(),
        name: item.name.clone(),
        amount_text: format!("{:.2}", item.amount),
        ends_at_text: item.ends_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        revision: item.revision.to_string(),
        approved_text: format!("{}", item.approved),
        source: item.source.clone(),
        category: item.category.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_limit_allocation_views(items: &[TieredLimitAllocation]) -> Vec<TieredLimitAllocationView> {
    let mut views: Vec<TieredLimitAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_limit_allocation_view(item));
    }
    views
}

pub fn sum_tiered_limit_allocation_view(view: &TieredLimitAllocationView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_amount + (parsed_ends_at as f64) + parsed_maximum)
}
