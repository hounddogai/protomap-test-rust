use super::*;

use acme_kit::text;

/// How screens and exports show a capped overage summary.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedOverageSummaryView {
    pub code: String,
    pub label: String,
    pub attempts: String,
    pub quantity: String,
    pub ceiling: String,
    pub visible_text: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_capped_overage_summary_view(item: &CappedOverageSummary) -> CappedOverageSummaryView {
    CappedOverageSummaryView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 40),
        attempts: item.attempts.to_string(),
        quantity: item.quantity.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        visible_text: format!("{}", item.visible),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_overage_summary_views(items: &[CappedOverageSummary]) -> Vec<CappedOverageSummaryView> {
    let mut views: Vec<CappedOverageSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_overage_summary_view(item));
    }
    views
}

pub fn sum_capped_overage_summary_view(view: &CappedOverageSummaryView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_quantity as f64) + parsed_ceiling)
}
