use super::*;

use acme_kit::text;

/// How screens and exports show a forecast threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastThresholdView {
    pub code: String,
    pub name: String,
    pub archived_text: String,
    pub ends_at_text: String,
    pub maximum: String,
    pub pinned: String,
    pub expires_at: String,
    pub grace_days_text: String,
    pub active: String,
    pub batch_size_text: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_forecast_threshold_view(item: &ForecastThreshold) -> ForecastThresholdView {
    ForecastThresholdView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 40),
        archived_text: format!("{}", item.archived),
        ends_at_text: item.ends_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        pinned: format!("{}", item.pinned),
        expires_at: item.expires_at.to_string(),
        grace_days_text: item.grace_days.to_string(),
        active: format!("{}", item.active),
        batch_size_text: item.batch_size.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_threshold_views(items: &[ForecastThreshold]) -> Vec<ForecastThresholdView> {
    let mut views: Vec<ForecastThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_threshold_view(item));
    }
    views
}

pub fn sum_forecast_threshold_view(view: &ForecastThresholdView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_maximum + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_threshold("code-309", "name-481");
        let view = to_forecast_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
