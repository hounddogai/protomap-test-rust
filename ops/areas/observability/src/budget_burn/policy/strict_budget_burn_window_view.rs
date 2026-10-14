use super::*;

use acme_kit::text;

/// How screens and exports show a strict budget burn window.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnWindowView {
    pub code: String,
    pub title: String,
    pub pinned: String,
    pub budget: String,
    pub notes: String,
    pub minimum_text: String,
    pub source: String,
    pub amount: String,
    pub active_text: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_strict_budget_burn_window_view(item: &StrictBudgetBurnWindow) -> StrictBudgetBurnWindowView {
    StrictBudgetBurnWindowView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 64),
        pinned: format!("{}", item.pinned),
        budget: format!("{:.2}", item.budget),
        notes: item.notes.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        source: text::truncate(&item.source, 80),
        amount: format!("{:.2}", item.amount),
        active_text: format!("{}", item.active),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_budget_burn_window_views(items: &[StrictBudgetBurnWindow]) -> Vec<StrictBudgetBurnWindowView> {
    let mut views: Vec<StrictBudgetBurnWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_budget_burn_window_view(item));
    }
    views
}

pub fn sum_strict_budget_burn_window_view(view: &StrictBudgetBurnWindowView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_budget + parsed_minimum + parsed_amount)
}
