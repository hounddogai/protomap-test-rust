use super::*;

use acme_kit::text;

/// How screens and exports show a rule metric.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleMetricView {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub quota_units: String,
    pub cost: String,
    pub limit: String,
    pub grace_days: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_rule_metric_view(item: &RuleMetric) -> RuleMetricView {
    RuleMetricView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        cluster: item.cluster.clone(),
        quota_units: item.quota_units.to_string(),
        cost: format!("{:.2}", item.cost),
        limit: item.limit.to_string(),
        grace_days: item.grace_days.to_string(),
        namespace: text::truncate(&item.namespace, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_metric_views(items: &[RuleMetric]) -> Vec<RuleMetricView> {
    let mut views: Vec<RuleMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_metric_view(item));
    }
    views
}

pub fn sum_rule_metric_view(view: &RuleMetricView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_cost + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rule_metric("code-399", "label-484");
        let view = to_rule_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rule_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
