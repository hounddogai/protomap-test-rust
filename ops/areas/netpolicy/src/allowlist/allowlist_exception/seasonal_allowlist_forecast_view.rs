use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal allowlist forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistForecastView {
    pub code: String,
    pub title_text: String,
    pub pinned_text: String,
    pub country: String,
    pub score: String,
    pub burst: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_seasonal_allowlist_forecast_view(item: &SeasonalAllowlistForecast) -> SeasonalAllowlistForecastView {
    SeasonalAllowlistForecastView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        pinned_text: format!("{}", item.pinned),
        country: text::truncate(&item.country, 80),
        score: format!("{:.2}", item.score),
        burst: item.burst.to_string(),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_allowlist_forecast_views(items: &[SeasonalAllowlistForecast]) -> Vec<SeasonalAllowlistForecastView> {
    let mut views: Vec<SeasonalAllowlistForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_allowlist_forecast_view(item));
    }
    views
}

pub fn sum_seasonal_allowlist_forecast_view(view: &SeasonalAllowlistForecastView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_score + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_allowlist_forecast("code-289", "title-93");
        let view = to_seasonal_allowlist_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_allowlist_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
