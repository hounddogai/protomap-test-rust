use super::*;

use acme_kit::text;

/// How screens and exports show a legacy rollout forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutForecastView {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub batch_size_text: String,
    pub minimum_text: String,
    pub score: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_legacy_rollout_forecast_view(item: &LegacyRolloutForecast) -> LegacyRolloutForecastView {
    LegacyRolloutForecastView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        issuer: text::truncate(&item.issuer, 120),
        batch_size_text: item.batch_size.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        score: format!("{:.2}", item.score),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_rollout_forecast_views(items: &[LegacyRolloutForecast]) -> Vec<LegacyRolloutForecastView> {
    let mut views: Vec<LegacyRolloutForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_rollout_forecast_view(item));
    }
    views
}

pub fn sum_legacy_rollout_forecast_view(view: &LegacyRolloutForecastView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_minimum + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_rollout_forecast("code-254", "title-347");
        let view = to_legacy_rollout_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_rollout_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
