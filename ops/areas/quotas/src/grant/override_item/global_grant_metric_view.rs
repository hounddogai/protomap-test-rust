use super::*;

use acme_kit::text;

/// How screens and exports show a global grant metric.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalGrantMetricView {
    pub code: String,
    pub name: String,
    pub updated_at: String,
    pub segment: String,
    pub ttl_seconds_text: String,
    pub summary_text: String,
    pub unit: String,
    pub weight: String,
    pub timezone: String,
    pub namespace_text: String,
    pub status_label: String,
}

pub fn to_global_grant_metric_view(item: &GlobalGrantMetric) -> GlobalGrantMetricView {
    GlobalGrantMetricView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        updated_at: item.updated_at.to_string(),
        segment: item.segment.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        summary_text: text::truncate(&item.summary, 120),
        unit: text::truncate(&item.unit, 80),
        weight: format!("{:.2}", item.weight),
        timezone: item.timezone.clone(),
        namespace_text: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_grant_metric_views(items: &[GlobalGrantMetric]) -> Vec<GlobalGrantMetricView> {
    let mut views: Vec<GlobalGrantMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_grant_metric_view(item));
    }
    views
}

pub fn sum_global_grant_metric_view(view: &GlobalGrantMetricView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_ttl_seconds as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_grant_metric("code-397", "name-491");
        let view = to_global_grant_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_grant_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
