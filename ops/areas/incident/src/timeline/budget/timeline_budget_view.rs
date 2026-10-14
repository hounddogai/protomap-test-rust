use super::*;

use acme_kit::text;

/// How screens and exports show a timeline budget.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineBudgetView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub margin: String,
    pub window_days: String,
    pub description: String,
    pub grace_days: String,
    pub key_id_text: String,
    pub audience_text: String,
    pub status_label: String,
}

pub fn to_timeline_budget_view(item: &TimelineBudget) -> TimelineBudgetView {
    TimelineBudgetView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        budget: format!("{:.2}", item.budget),
        margin: format!("{:.2}", item.margin),
        window_days: item.window_days.to_string(),
        description: text::truncate(&item.description, 120),
        grace_days: item.grace_days.to_string(),
        key_id_text: text::truncate(&item.key_id, 64),
        audience_text: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_budget_views(items: &[TimelineBudget]) -> Vec<TimelineBudgetView> {
    let mut views: Vec<TimelineBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_budget_view(item));
    }
    views
}

pub fn sum_timeline_budget_view(view: &TimelineBudgetView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_budget + parsed_margin + (parsed_window_days as f64))
}
