use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal budget burn plan.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnPlanView {
    pub code_text: String,
    pub label: String,
    pub score: String,
    pub retries_text: String,
    pub version: String,
    pub archived_text: String,
    pub currency: String,
    pub fingerprint_text: String,
    pub phone: String,
    pub notes: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_seasonal_budget_burn_plan_view(item: &SeasonalBudgetBurnPlan) -> SeasonalBudgetBurnPlanView {
    SeasonalBudgetBurnPlanView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        score: format!("{:.2}", item.score),
        retries_text: item.retries.to_string(),
        version: item.version.to_string(),
        archived_text: format!("{}", item.archived),
        currency: item.currency.clone(),
        fingerprint_text: item.fingerprint.clone(),
        phone: item.phone.clone(),
        notes: item.notes.clone(),
        segment: text::truncate(&item.segment, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_budget_burn_plan_views(items: &[SeasonalBudgetBurnPlan]) -> Vec<SeasonalBudgetBurnPlanView> {
    let mut views: Vec<SeasonalBudgetBurnPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_budget_burn_plan_view(item));
    }
    views
}

pub fn sum_seasonal_budget_burn_plan_view(view: &SeasonalBudgetBurnPlanView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_score + (parsed_retries as f64) + (parsed_version as f64))
}
