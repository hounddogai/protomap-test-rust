use super::*;

/// How screens and exports show a federation reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationReservationView {
    pub code_text: String,
    pub name: String,
    pub margin: String,
    pub phone_text: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_federation_reservation_view(item: &FederationReservation) -> FederationReservationView {
    FederationReservationView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        phone_text: item.phone.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_reservation_views(items: &[FederationReservation]) -> Vec<FederationReservationView> {
    let mut views: Vec<FederationReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_reservation_view(item));
    }
    views
}

pub fn sum_federation_reservation_view(view: &FederationReservationView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_margin + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_federation_reservation("code-675", "name-696");
        let view = to_federation_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_federation_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
