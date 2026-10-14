use super::*;

use acme_kit::text;

/// How screens and exports show a rolling budget burn metric.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnMetricView {
    pub code: String,
    pub title: String,
    pub ttl_seconds_text: String,
    pub window_days: String,
    pub amount: String,
    pub visible: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_rolling_budget_burn_metric_view(item: &RollingBudgetBurnMetric) -> RollingBudgetBurnMetricView {
    RollingBudgetBurnMetricView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        window_days: item.window_days.to_string(),
        amount: format!("{:.2}", item.amount),
        visible: format!("{}", item.visible),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_budget_burn_metric_views(items: &[RollingBudgetBurnMetric]) -> Vec<RollingBudgetBurnMetricView> {
    let mut views: Vec<RollingBudgetBurnMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_budget_burn_metric_view(item));
    }
    views
}

pub fn sum_rolling_budget_burn_metric_view(view: &RollingBudgetBurnMetricView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_window_days as f64) + parsed_amount)
}
