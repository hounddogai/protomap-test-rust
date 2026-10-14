use super::*;

use acme_kit::text;

/// How screens and exports show a legacy silence budget.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceBudgetView {
    pub code: String,
    pub title_text: String,
    pub capacity_text: String,
    pub rate: String,
    pub label: String,
    pub checksum_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_legacy_silence_budget_view(item: &LegacySilenceBudget) -> LegacySilenceBudgetView {
    LegacySilenceBudgetView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        capacity_text: item.capacity.to_string(),
        rate: format!("{:.2}", item.rate),
        label: item.label.clone(),
        checksum_text: item.checksum.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_silence_budget_views(items: &[LegacySilenceBudget]) -> Vec<LegacySilenceBudgetView> {
    let mut views: Vec<LegacySilenceBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_silence_budget_view(item));
    }
    views
}

pub fn sum_legacy_silence_budget_view(view: &LegacySilenceBudgetView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_rate + parsed_amount)
}
