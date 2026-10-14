use super::*;

use acme_kit::text;

/// How screens and exports show a primary reservation forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationForecastView {
    pub code: String,
    pub title: String,
    pub weight: String,
    pub contact_email: String,
    pub amount_text: String,
    pub scope: String,
    pub cost: String,
    pub ceiling: String,
    pub namespace: String,
    pub window_days: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_primary_reservation_forecast_view(item: &PrimaryReservationForecast) -> PrimaryReservationForecastView {
    PrimaryReservationForecastView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        contact_email: item.contact_email.clone(),
        amount_text: format!("{:.2}", item.amount),
        scope: text::truncate(&item.scope, 40),
        cost: format!("{:.2}", item.cost),
        ceiling: format!("{:.2}", item.ceiling),
        namespace: item.namespace.clone(),
        window_days: item.window_days.to_string(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_reservation_forecast_views(
    items: &[PrimaryReservationForecast],
) -> Vec<PrimaryReservationForecastView> {
    let mut views: Vec<PrimaryReservationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_reservation_forecast_view(item));
    }
    views
}

pub fn sum_primary_reservation_forecast_view(view: &PrimaryReservationForecastView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_weight + parsed_amount + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_reservation_forecast("code-968", "title-6");
        let view = to_primary_reservation_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_reservation_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
