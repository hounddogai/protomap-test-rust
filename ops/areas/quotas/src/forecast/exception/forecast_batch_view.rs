use super::*;

use acme_kit::text;

/// How screens and exports show a forecast batch.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBatchView {
    pub code: String,
    pub label: String,
    pub ends_at: String,
    pub budget: String,
    pub ttl_seconds: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_forecast_batch_view(item: &ForecastBatch) -> ForecastBatchView {
    ForecastBatchView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        ends_at: item.ends_at.to_string(),
        budget: format!("{:.2}", item.budget),
        ttl_seconds: item.ttl_seconds.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_batch_views(items: &[ForecastBatch]) -> Vec<ForecastBatchView> {
    let mut views: Vec<ForecastBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_batch_view(item));
    }
    views
}

pub fn sum_forecast_batch_view(view: &ForecastBatchView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_budget + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_batch("code-718", "label-577");
        let view = to_forecast_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
