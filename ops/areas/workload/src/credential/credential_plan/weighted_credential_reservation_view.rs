use super::*;

/// How screens and exports show a weighted credential reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialReservationView {
    pub code_text: String,
    pub title: String,
    pub audience: String,
    pub active: String,
    pub region: String,
    pub country: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_weighted_credential_reservation_view(
    item: &WeightedCredentialReservation,
) -> WeightedCredentialReservationView {
    WeightedCredentialReservationView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        audience: item.audience.clone(),
        active: format!("{}", item.active),
        region: item.region.clone(),
        country: item.country.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_credential_reservation_views(
    items: &[WeightedCredentialReservation],
) -> Vec<WeightedCredentialReservationView> {
    let mut views: Vec<WeightedCredentialReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_credential_reservation_view(item));
    }
    views
}

pub fn sum_weighted_credential_reservation_view(view: &WeightedCredentialReservationView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_credential_reservation("code-644", "title-516");
        let view = to_weighted_credential_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_credential_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
