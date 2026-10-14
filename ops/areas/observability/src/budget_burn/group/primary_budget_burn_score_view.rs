use super::*;

use acme_kit::text;

/// How screens and exports show a primary budget burn score.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnScoreView {
    pub code: String,
    pub label: String,
    pub floor_value: String,
    pub contact_email: String,
    pub starts_at: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_primary_budget_burn_score_view(item: &PrimaryBudgetBurnScore) -> PrimaryBudgetBurnScoreView {
    PrimaryBudgetBurnScoreView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        contact_email: item.contact_email.clone(),
        starts_at: item.starts_at.to_string(),
        currency_text: text::truncate(&item.currency, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_budget_burn_score_views(items: &[PrimaryBudgetBurnScore]) -> Vec<PrimaryBudgetBurnScoreView> {
    let mut views: Vec<PrimaryBudgetBurnScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_budget_burn_score_view(item));
    }
    views
}

pub fn sum_primary_budget_burn_score_view(view: &PrimaryBudgetBurnScoreView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_floor_value + (parsed_starts_at as f64))
}
