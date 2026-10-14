use super::*;

use acme_kit::text;

/// How screens and exports show a reservation forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationForecastView {
    pub code: String,
    pub label_text: String,
    pub limit: String,
    pub contact_email: String,
    pub quota_units: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_reservation_forecast_view(item: &ReservationForecast) -> ReservationForecastView {
    ReservationForecastView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        limit: item.limit.to_string(),
        contact_email: item.contact_email.clone(),
        quota_units: item.quota_units.to_string(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_forecast_views(items: &[ReservationForecast]) -> Vec<ReservationForecastView> {
    let mut views: Vec<ReservationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_forecast_view(item));
    }
    views
}

pub fn sum_reservation_forecast_view(view: &ReservationForecastView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_quota_units as f64) + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_forecast("code-951", "label-932");
        let view = to_reservation_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
