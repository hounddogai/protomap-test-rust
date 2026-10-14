use super::*;

use acme_kit::text;

/// How screens and exports show a lenient reservation forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationForecastView {
    pub code_text: String,
    pub name_text: String,
    pub category: String,
    pub rotation_days: String,
    pub audience_text: String,
    pub archived: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_lenient_reservation_forecast_view(item: &LenientReservationForecast) -> LenientReservationForecastView {
    LenientReservationForecastView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        category: text::truncate(&item.category, 80),
        rotation_days: item.rotation_days.to_string(),
        audience_text: text::truncate(&item.audience, 80),
        archived: format!("{}", item.archived),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_reservation_forecast_views(
    items: &[LenientReservationForecast],
) -> Vec<LenientReservationForecastView> {
    let mut views: Vec<LenientReservationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_reservation_forecast_view(item));
    }
    views
}

pub fn sum_lenient_reservation_forecast_view(view: &LenientReservationForecastView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_rotation_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_reservation_forecast("code-937", "name-644");
        let view = to_lenient_reservation_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_reservation_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
