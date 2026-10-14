use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyForecastView {
    pub code: String,
    pub label: String,
    pub ttl_seconds: String,
    pub unit: String,
    pub floor_value: String,
    pub pinned_text: String,
    pub retries: String,
    pub visible_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_ttl_policy_forecast_view(item: &TtlPolicyForecast) -> TtlPolicyForecastView {
    TtlPolicyForecastView {
        code: item.code.clone(),
        label: item.label.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        unit: text::truncate(&item.unit, 80),
        floor_value: format!("{:.2}", item.floor_value),
        pinned_text: format!("{}", item.pinned),
        retries: item.retries.to_string(),
        visible_text: format!("{}", item.visible),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_forecast_views(items: &[TtlPolicyForecast]) -> Vec<TtlPolicyForecastView> {
    let mut views: Vec<TtlPolicyForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_forecast_view(item));
    }
    views
}

pub fn sum_ttl_policy_forecast_view(view: &TtlPolicyForecastView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_floor_value + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_forecast("code-66", "label-863");
        let view = to_ttl_policy_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
