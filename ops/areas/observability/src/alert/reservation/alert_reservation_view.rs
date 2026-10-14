use super::*;

/// How screens and exports show a alert reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertReservationView {
    pub code: String,
    pub name: String,
    pub region: String,
    pub ratio: String,
    pub locale: String,
    pub floor_value: String,
    pub approved: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_alert_reservation_view(item: &AlertReservation) -> AlertReservationView {
    AlertReservationView {
        code: item.code.clone(),
        name: item.name.clone(),
        region: item.region.clone(),
        ratio: format!("{:.2}", item.ratio),
        locale: item.locale.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        approved: format!("{}", item.approved),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_reservation_views(items: &[AlertReservation]) -> Vec<AlertReservationView> {
    let mut views: Vec<AlertReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_reservation_view(item));
    }
    views
}

pub fn sum_alert_reservation_view(view: &AlertReservationView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_ratio + parsed_floor_value + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_reservation("code-914", "name-139");
        let view = to_alert_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
