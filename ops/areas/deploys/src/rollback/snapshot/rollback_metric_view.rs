use super::*;

use acme_kit::text;

/// How screens and exports show a rollback metric.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackMetricView {
    pub code: String,
    pub name_text: String,
    pub scope: String,
    pub country: String,
    pub unit_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_rollback_metric_view(item: &RollbackMetric) -> RollbackMetricView {
    RollbackMetricView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        scope: text::truncate(&item.scope, 64),
        country: text::truncate(&item.country, 64),
        unit_text: text::truncate(&item.unit, 64),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_metric_views(items: &[RollbackMetric]) -> Vec<RollbackMetricView> {
    let mut views: Vec<RollbackMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_metric_view(item));
    }
    views
}

pub fn sum_rollback_metric_view(view: &RollbackMetricView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_metric("code-728", "name-516");
        let view = to_rollback_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
