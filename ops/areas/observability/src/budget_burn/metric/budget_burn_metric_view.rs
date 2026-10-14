use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn metric.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMetricView {
    pub code: String,
    pub title_text: String,
    pub attempts: String,
    pub ttl_seconds: String,
    pub reviewer: String,
    pub country: String,
    pub approved_text: String,
    pub status_label: String,
}

pub fn to_budget_burn_metric_view(item: &BudgetBurnMetric) -> BudgetBurnMetricView {
    BudgetBurnMetricView {
        code: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        attempts: item.attempts.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        reviewer: item.reviewer.clone(),
        country: text::truncate(&item.country, 80),
        approved_text: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_metric_views(items: &[BudgetBurnMetric]) -> Vec<BudgetBurnMetricView> {
    let mut views: Vec<BudgetBurnMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_metric_view(item));
    }
    views
}

pub fn sum_budget_burn_metric_view(view: &BudgetBurnMetricView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_metric("code-843", "title-421");
        let view = to_budget_burn_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
