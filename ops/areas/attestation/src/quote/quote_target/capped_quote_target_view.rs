use super::*;

use acme_kit::text;

/// How screens and exports show a capped quote target.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteTargetView {
    pub code: String,
    pub label_text: String,
    pub owner_id_text: String,
    pub budget_text: String,
    pub status_label: String,
}

pub fn to_capped_quote_target_view(item: &CappedQuoteTarget) -> CappedQuoteTargetView {
    CappedQuoteTargetView {
        code: text::truncate(&item.code, 80),
        label_text: item.label.clone(),
        owner_id_text: text::truncate(&item.owner_id, 40),
        budget_text: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_quote_target_views(items: &[CappedQuoteTarget]) -> Vec<CappedQuoteTargetView> {
    let mut views: Vec<CappedQuoteTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_quote_target_view(item));
    }
    views
}

pub fn sum_capped_quote_target_view(view: &CappedQuoteTargetView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}
