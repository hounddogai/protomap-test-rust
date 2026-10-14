use super::*;

/// How screens and exports show a credential reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialReservationView {
    pub code_text: String,
    pub name_text: String,
    pub region: String,
    pub minimum: String,
    pub quantity: String,
    pub grace_days: String,
    pub host: String,
    pub notes: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_credential_reservation_view(item: &CredentialReservation) -> CredentialReservationView {
    CredentialReservationView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        region: item.region.clone(),
        minimum: format!("{:.2}", item.minimum),
        quantity: item.quantity.to_string(),
        grace_days: item.grace_days.to_string(),
        host: item.host.clone(),
        notes: item.notes.clone(),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_reservation_views(items: &[CredentialReservation]) -> Vec<CredentialReservationView> {
    let mut views: Vec<CredentialReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_reservation_view(item));
    }
    views
}

pub fn sum_credential_reservation_view(view: &CredentialReservationView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_minimum + (parsed_quantity as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_reservation("code-67", "name-253");
        let view = to_credential_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
