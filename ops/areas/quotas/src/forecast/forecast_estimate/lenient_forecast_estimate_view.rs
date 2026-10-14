use super::*;

use acme_kit::text;

/// How screens and exports show a lenient forecast estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientForecastEstimateView {
    pub code: String,
    pub label_text: String,
    pub limit_text: String,
    pub floor_value_text: String,
    pub scope: String,
    pub title_text: String,
    pub channel: String,
    pub cost: String,
    pub grace_days: String,
    pub pinned: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_lenient_forecast_estimate_view(item: &LenientForecastEstimate) -> LenientForecastEstimateView {
    LenientForecastEstimateView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        limit_text: item.limit.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        scope: item.scope.clone(),
        title_text: text::truncate(&item.title, 120),
        channel: item.channel.clone(),
        cost: format!("{:.2}", item.cost),
        grace_days: item.grace_days.to_string(),
        pinned: format!("{}", item.pinned),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_forecast_estimate_views(items: &[LenientForecastEstimate]) -> Vec<LenientForecastEstimateView> {
    let mut views: Vec<LenientForecastEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_forecast_estimate_view(item));
    }
    views
}

pub fn sum_lenient_forecast_estimate_view(view: &LenientForecastEstimateView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_limit as f64 + parsed_floor_value + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_forecast_estimate("code-998", "label-329");
        let view = to_lenient_forecast_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_forecast_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
