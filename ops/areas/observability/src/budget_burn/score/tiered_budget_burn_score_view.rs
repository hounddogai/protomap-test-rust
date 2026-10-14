use super::*;

use acme_kit::text;

/// How screens and exports show a tiered budget burn score.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnScoreView {
    pub code_text: String,
    pub label: String,
    pub score: String,
    pub attempts: String,
    pub namespace: String,
    pub audience: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_tiered_budget_burn_score_view(item: &TieredBudgetBurnScore) -> TieredBudgetBurnScoreView {
    TieredBudgetBurnScoreView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 64),
        score: format!("{:.2}", item.score),
        attempts: item.attempts.to_string(),
        namespace: item.namespace.clone(),
        audience: item.audience.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_budget_burn_score_views(items: &[TieredBudgetBurnScore]) -> Vec<TieredBudgetBurnScoreView> {
    let mut views: Vec<TieredBudgetBurnScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_budget_burn_score_view(item));
    }
    views
}

pub fn sum_tiered_budget_burn_score_view(view: &TieredBudgetBurnScoreView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_score + (parsed_attempts as f64) + (parsed_priority as f64))
}
