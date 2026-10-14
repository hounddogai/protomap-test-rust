use super::*;

/// How screens and exports show a weighted alert reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertReservationView {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub ceiling_text: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_weighted_alert_reservation_view(item: &WeightedAlertReservation) -> WeightedAlertReservationView {
    WeightedAlertReservationView {
        code: item.code.clone(),
        name: item.name.clone(),
        scope: item.scope.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_alert_reservation_views(items: &[WeightedAlertReservation]) -> Vec<WeightedAlertReservationView> {
    let mut views: Vec<WeightedAlertReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_alert_reservation_view(item));
    }
    views
}

pub fn sum_weighted_alert_reservation_view(view: &WeightedAlertReservationView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_ceiling + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_alert_reservation("code-361", "name-824");
        let view = to_weighted_alert_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_alert_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
