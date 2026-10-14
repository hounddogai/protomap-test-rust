use super::*;

use acme_kit::text;

/// How screens and exports show a legacy nonce forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyNonceForecastView {
    pub code_text: String,
    pub title: String,
    pub window_days: String,
    pub weight: String,
    pub external_ref: String,
    pub grace_days: String,
    pub namespace: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_legacy_nonce_forecast_view(item: &LegacyNonceForecast) -> LegacyNonceForecastView {
    LegacyNonceForecastView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        window_days: item.window_days.to_string(),
        weight: format!("{:.2}", item.weight),
        external_ref: item.external_ref.clone(),
        grace_days: item.grace_days.to_string(),
        namespace: text::truncate(&item.namespace, 80),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_nonce_forecast_views(items: &[LegacyNonceForecast]) -> Vec<LegacyNonceForecastView> {
    let mut views: Vec<LegacyNonceForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_nonce_forecast_view(item));
    }
    views
}

pub fn sum_legacy_nonce_forecast_view(view: &LegacyNonceForecastView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_weight + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_nonce_forecast("code-803", "title-389");
        let view = to_legacy_nonce_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_nonce_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
