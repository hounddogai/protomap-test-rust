use super::*;

use acme_kit::text;

/// How screens and exports show a batched runbook forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRunbookForecastView {
    pub code: String,
    pub label: String,
    pub grace_days: String,
    pub name: String,
    pub currency_text: String,
    pub weight: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_batched_runbook_forecast_view(item: &BatchedRunbookForecast) -> BatchedRunbookForecastView {
    BatchedRunbookForecastView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 80),
        grace_days: item.grace_days.to_string(),
        name: item.name.clone(),
        currency_text: item.currency.clone(),
        weight: format!("{:.2}", item.weight),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_runbook_forecast_views(items: &[BatchedRunbookForecast]) -> Vec<BatchedRunbookForecastView> {
    let mut views: Vec<BatchedRunbookForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_runbook_forecast_view(item));
    }
    views
}

pub fn sum_batched_runbook_forecast_view(view: &BatchedRunbookForecastView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_weight + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_runbook_forecast("code-642", "label-938");
        let view = to_batched_runbook_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_runbook_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
