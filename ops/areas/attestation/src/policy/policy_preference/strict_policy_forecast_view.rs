use super::*;

/// How screens and exports show a strict policy forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicyForecastView {
    pub code: String,
    pub label_text: String,
    pub host: String,
    pub description: String,
    pub quantity_text: String,
    pub active_text: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_strict_policy_forecast_view(item: &StrictPolicyForecast) -> StrictPolicyForecastView {
    StrictPolicyForecastView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        host: item.host.clone(),
        description: item.description.clone(),
        quantity_text: item.quantity.to_string(),
        active_text: format!("{}", item.active),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_policy_forecast_views(items: &[StrictPolicyForecast]) -> Vec<StrictPolicyForecastView> {
    let mut views: Vec<StrictPolicyForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_policy_forecast_view(item));
    }
    views
}

pub fn sum_strict_policy_forecast_view(view: &StrictPolicyForecastView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_cost)
}
