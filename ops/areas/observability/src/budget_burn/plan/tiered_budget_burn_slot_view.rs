use super::*;

use acme_kit::text;

/// How screens and exports show a tiered budget burn slot.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnSlotView {
    pub code: String,
    pub label: String,
    pub burst: String,
    pub ceiling: String,
    pub summary_text: String,
    pub host: String,
    pub rotation_days_text: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_tiered_budget_burn_slot_view(item: &TieredBudgetBurnSlot) -> TieredBudgetBurnSlotView {
    TieredBudgetBurnSlotView {
        code: item.code.clone(),
        label: item.label.clone(),
        burst: item.burst.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        summary_text: text::truncate(&item.summary, 80),
        host: item.host.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_budget_burn_slot_views(items: &[TieredBudgetBurnSlot]) -> Vec<TieredBudgetBurnSlotView> {
    let mut views: Vec<TieredBudgetBurnSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_budget_burn_slot_view(item));
    }
    views
}

pub fn sum_tiered_budget_burn_slot_view(view: &TieredBudgetBurnSlotView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_burst as f64 + parsed_ceiling + (parsed_rotation_days as f64))
}
