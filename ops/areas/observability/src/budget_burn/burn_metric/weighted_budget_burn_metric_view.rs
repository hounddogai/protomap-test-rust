use super::*;

/// How screens and exports show a weighted budget burn metric.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnMetricView {
    pub code: String,
    pub title: String,
    pub minimum: String,
    pub revision: String,
    pub active: String,
    pub floor_value_text: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_weighted_budget_burn_metric_view(item: &WeightedBudgetBurnMetric) -> WeightedBudgetBurnMetricView {
    WeightedBudgetBurnMetricView {
        code: item.code.clone(),
        title: item.title.clone(),
        minimum: format!("{:.2}", item.minimum),
        revision: item.revision.to_string(),
        active: format!("{}", item.active),
        floor_value_text: format!("{:.2}", item.floor_value),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_budget_burn_metric_views(items: &[WeightedBudgetBurnMetric]) -> Vec<WeightedBudgetBurnMetricView> {
    let mut views: Vec<WeightedBudgetBurnMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_budget_burn_metric_view(item));
    }
    views
}

pub fn sum_weighted_budget_burn_metric_view(view: &WeightedBudgetBurnMetricView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_minimum + (parsed_revision as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_budget_burn_metric("code-628", "title-794");
        let view = to_weighted_budget_burn_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_budget_burn_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
