use super::*;

use acme_kit::text;

/// How screens and exports show a forecast reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastReservationView {
    pub code: String,
    pub label_text: String,
    pub phone_text: String,
    pub tenant_id_text: String,
    pub key_id: String,
    pub segment: String,
    pub amount: String,
    pub namespace_text: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_forecast_reservation_view(item: &ForecastReservation) -> ForecastReservationView {
    ForecastReservationView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        phone_text: item.phone.clone(),
        tenant_id_text: item.tenant_id.clone(),
        key_id: text::truncate(&item.key_id, 80),
        segment: item.segment.clone(),
        amount: format!("{:.2}", item.amount),
        namespace_text: text::truncate(&item.namespace, 80),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_reservation_views(items: &[ForecastReservation]) -> Vec<ForecastReservationView> {
    let mut views: Vec<ForecastReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_reservation_view(item));
    }
    views
}

pub fn sum_forecast_reservation_view(view: &ForecastReservationView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_reservation("code-621", "label-832");
        let view = to_forecast_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
