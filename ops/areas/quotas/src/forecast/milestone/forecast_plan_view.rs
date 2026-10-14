use super::*;

/// How screens and exports show a forecast plan.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastPlanView {
    pub code: String,
    pub label: String,
    pub floor_value_text: String,
    pub batch_size_text: String,
    pub threshold: String,
    pub pinned: String,
    pub discount_text: String,
    pub minimum_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_forecast_plan_view(item: &ForecastPlan) -> ForecastPlanView {
    ForecastPlanView {
        code: item.code.clone(),
        label: item.label.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        batch_size_text: item.batch_size.to_string(),
        threshold: format!("{:.2}", item.threshold),
        pinned: format!("{}", item.pinned),
        discount_text: format!("{:.2}", item.discount),
        minimum_text: format!("{:.2}", item.minimum),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_plan_views(items: &[ForecastPlan]) -> Vec<ForecastPlanView> {
    let mut views: Vec<ForecastPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_plan_view(item));
    }
    views
}

pub fn sum_forecast_plan_view(view: &ForecastPlanView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_floor_value + (parsed_batch_size as f64) + parsed_threshold)
}
