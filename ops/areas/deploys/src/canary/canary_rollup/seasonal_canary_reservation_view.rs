use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal canary reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryReservationView {
    pub code: String,
    pub name: String,
    pub visible: String,
    pub active: String,
    pub checksum: String,
    pub amount: String,
    pub minimum: String,
    pub revision: String,
    pub weight: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_seasonal_canary_reservation_view(item: &SeasonalCanaryReservation) -> SeasonalCanaryReservationView {
    SeasonalCanaryReservationView {
        code: item.code.clone(),
        name: item.name.clone(),
        visible: format!("{}", item.visible),
        active: format!("{}", item.active),
        checksum: text::truncate(&item.checksum, 40),
        amount: format!("{:.2}", item.amount),
        minimum: format!("{:.2}", item.minimum),
        revision: item.revision.to_string(),
        weight: format!("{:.2}", item.weight),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_canary_reservation_views(items: &[SeasonalCanaryReservation]) -> Vec<SeasonalCanaryReservationView> {
    let mut views: Vec<SeasonalCanaryReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_canary_reservation_view(item));
    }
    views
}

pub fn sum_seasonal_canary_reservation_view(view: &SeasonalCanaryReservationView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_amount + parsed_minimum + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_canary_reservation("code-216", "name-789");
        let view = to_seasonal_canary_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_canary_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
