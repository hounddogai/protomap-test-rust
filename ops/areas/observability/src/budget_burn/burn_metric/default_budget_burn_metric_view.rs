use super::*;

use acme_kit::text;

/// How screens and exports show a default budget burn metric.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBudgetBurnMetricView {
    pub code: String,
    pub label: String,
    pub window_days: String,
    pub floor_value_text: String,
    pub contact_email: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_default_budget_burn_metric_view(item: &DefaultBudgetBurnMetric) -> DefaultBudgetBurnMetricView {
    DefaultBudgetBurnMetricView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        window_days: item.window_days.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        contact_email: item.contact_email.clone(),
        currency: text::truncate(&item.currency, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_budget_burn_metric_views(items: &[DefaultBudgetBurnMetric]) -> Vec<DefaultBudgetBurnMetricView> {
    let mut views: Vec<DefaultBudgetBurnMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_budget_burn_metric_view(item));
    }
    views
}

pub fn sum_default_budget_burn_metric_view(view: &DefaultBudgetBurnMetricView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_budget_burn_metric("code-117", "label-907");
        let view = to_default_budget_burn_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_budget_burn_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
