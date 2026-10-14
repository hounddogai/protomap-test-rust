use super::*;

use acme_kit::text;

/// How screens and exports show a weighted forecast metric.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMetricView {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub principal_text: String,
    pub source: String,
    pub key_id_text: String,
    pub cipher: String,
    pub rotation_days: String,
    pub namespace_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_weighted_forecast_metric_view(item: &WeightedForecastMetric) -> WeightedForecastMetricView {
    WeightedForecastMetricView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        segment: text::truncate(&item.segment, 80),
        principal_text: item.principal.clone(),
        source: item.source.clone(),
        key_id_text: item.key_id.clone(),
        cipher: item.cipher.clone(),
        rotation_days: item.rotation_days.to_string(),
        namespace_text: item.namespace.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_forecast_metric_views(items: &[WeightedForecastMetric]) -> Vec<WeightedForecastMetricView> {
    let mut views: Vec<WeightedForecastMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_forecast_metric_view(item));
    }
    views
}

pub fn sum_weighted_forecast_metric_view(view: &WeightedForecastMetricView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_forecast_metric("code-986", "name-626");
        let view = to_weighted_forecast_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_forecast_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
