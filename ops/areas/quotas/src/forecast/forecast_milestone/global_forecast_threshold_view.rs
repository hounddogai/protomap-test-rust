use super::*;

use acme_kit::text;

/// How screens and exports show a global forecast threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastThresholdView {
    pub code: String,
    pub title: String,
    pub starts_at: String,
    pub score_text: String,
    pub category: String,
    pub external_ref: String,
    pub visible: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_global_forecast_threshold_view(item: &GlobalForecastThreshold) -> GlobalForecastThresholdView {
    GlobalForecastThresholdView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        starts_at: item.starts_at.to_string(),
        score_text: format!("{:.2}", item.score),
        category: item.category.clone(),
        external_ref: text::truncate(&item.external_ref, 80),
        visible: format!("{}", item.visible),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_forecast_threshold_views(items: &[GlobalForecastThreshold]) -> Vec<GlobalForecastThresholdView> {
    let mut views: Vec<GlobalForecastThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_forecast_threshold_view(item));
    }
    views
}

pub fn sum_global_forecast_threshold_view(view: &GlobalForecastThresholdView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_score + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_forecast_threshold("code-610", "title-257");
        let view = to_global_forecast_threshold_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_forecast_threshold_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
