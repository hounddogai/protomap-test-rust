use super::*;

use acme_kit::text;

/// How screens and exports show a canary forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryForecastView {
    pub code: String,
    pub label_text: String,
    pub version_text: String,
    pub key_id: String,
    pub notes_text: String,
    pub margin: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_canary_forecast_view(item: &CanaryForecast) -> CanaryForecastView {
    CanaryForecastView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        version_text: item.version.to_string(),
        key_id: item.key_id.clone(),
        notes_text: text::truncate(&item.notes, 120),
        margin: format!("{:.2}", item.margin),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_forecast_views(items: &[CanaryForecast]) -> Vec<CanaryForecastView> {
    let mut views: Vec<CanaryForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_forecast_view(item));
    }
    views
}

pub fn sum_canary_forecast_view(view: &CanaryForecastView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_version as f64 + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_forecast("code-446", "label-812");
        let view = to_canary_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
