use super::*;

use acme_kit::text;

/// How screens and exports show a capped dashboard metric.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardMetricView {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub duration_minutes: String,
    pub burst: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_capped_dashboard_metric_view(item: &CappedDashboardMetric) -> CappedDashboardMetricView {
    CappedDashboardMetricView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        external_ref: item.external_ref.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        burst: item.burst.to_string(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_dashboard_metric_views(items: &[CappedDashboardMetric]) -> Vec<CappedDashboardMetricView> {
    let mut views: Vec<CappedDashboardMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_dashboard_metric_view(item));
    }
    views
}

pub fn sum_capped_dashboard_metric_view(view: &CappedDashboardMetricView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_burst as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_dashboard_metric("code-622", "label-736");
        let view = to_capped_dashboard_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_dashboard_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
