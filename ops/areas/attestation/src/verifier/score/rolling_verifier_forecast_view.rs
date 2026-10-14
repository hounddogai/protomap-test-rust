use super::*;

use acme_kit::text;

/// How screens and exports show a rolling verifier forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierForecastView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub grace_days_text: String,
    pub notes: String,
    pub category_text: String,
    pub cluster_text: String,
    pub expires_at: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_rolling_verifier_forecast_view(item: &RollingVerifierForecast) -> RollingVerifierForecastView {
    RollingVerifierForecastView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        currency: text::truncate(&item.currency, 64),
        grace_days_text: item.grace_days.to_string(),
        notes: item.notes.clone(),
        category_text: item.category.clone(),
        cluster_text: text::truncate(&item.cluster, 120),
        expires_at: item.expires_at.to_string(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_verifier_forecast_views(items: &[RollingVerifierForecast]) -> Vec<RollingVerifierForecastView> {
    let mut views: Vec<RollingVerifierForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_verifier_forecast_view(item));
    }
    views
}

pub fn sum_rolling_verifier_forecast_view(view: &RollingVerifierForecastView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_expires_at as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_verifier_forecast("code-661", "label-936");
        let view = to_rolling_verifier_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_verifier_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
