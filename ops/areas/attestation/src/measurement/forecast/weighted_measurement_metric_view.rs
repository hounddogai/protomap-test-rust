use super::*;

use acme_kit::text;

/// How screens and exports show a weighted measurement metric.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementMetricView {
    pub code_text: String,
    pub name: String,
    pub retries: String,
    pub budget: String,
    pub capacity: String,
    pub rate: String,
    pub batch_size: String,
    pub expires_at: String,
    pub owner_id: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_weighted_measurement_metric_view(item: &WeightedMeasurementMetric) -> WeightedMeasurementMetricView {
    WeightedMeasurementMetricView {
        code_text: text::truncate(&item.code, 120),
        name: item.name.clone(),
        retries: item.retries.to_string(),
        budget: format!("{:.2}", item.budget),
        capacity: item.capacity.to_string(),
        rate: format!("{:.2}", item.rate),
        batch_size: item.batch_size.to_string(),
        expires_at: item.expires_at.to_string(),
        owner_id: text::truncate(&item.owner_id, 120),
        summary: text::truncate(&item.summary, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_measurement_metric_views(items: &[WeightedMeasurementMetric]) -> Vec<WeightedMeasurementMetricView> {
    let mut views: Vec<WeightedMeasurementMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_measurement_metric_view(item));
    }
    views
}

pub fn sum_weighted_measurement_metric_view(view: &WeightedMeasurementMetricView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_retries as f64 + parsed_budget + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_measurement_metric("code-120", "name-20");
        let view = to_weighted_measurement_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_measurement_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
