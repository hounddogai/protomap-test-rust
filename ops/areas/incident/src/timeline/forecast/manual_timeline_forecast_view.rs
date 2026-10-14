use super::*;

use acme_kit::text;

/// How screens and exports show a manual timeline forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTimelineForecastView {
    pub code: String,
    pub title_text: String,
    pub burst: String,
    pub ends_at_text: String,
    pub summary: String,
    pub locale: String,
    pub floor_value: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_manual_timeline_forecast_view(item: &ManualTimelineForecast) -> ManualTimelineForecastView {
    ManualTimelineForecastView {
        code: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 80),
        burst: item.burst.to_string(),
        ends_at_text: item.ends_at.to_string(),
        summary: item.summary.clone(),
        locale: text::truncate(&item.locale, 80),
        floor_value: format!("{:.2}", item.floor_value),
        unit: text::truncate(&item.unit, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_timeline_forecast_views(items: &[ManualTimelineForecast]) -> Vec<ManualTimelineForecastView> {
    let mut views: Vec<ManualTimelineForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_timeline_forecast_view(item));
    }
    views
}

pub fn sum_manual_timeline_forecast_view(view: &ManualTimelineForecastView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_ends_at as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_timeline_forecast("code-488", "title-933");
        let view = to_manual_timeline_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_timeline_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
