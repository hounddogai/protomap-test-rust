use super::*;

use acme_kit::text;

/// How screens and exports show a policy reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyReservationView {
    pub code_text: String,
    pub title_text: String,
    pub grace_days: String,
    pub minimum: String,
    pub currency: String,
    pub visible: String,
    pub maximum_text: String,
    pub port: String,
    pub retries_text: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_policy_reservation_view(item: &PolicyReservation) -> PolicyReservationView {
    PolicyReservationView {
        code_text: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 120),
        grace_days: item.grace_days.to_string(),
        minimum: format!("{:.2}", item.minimum),
        currency: item.currency.clone(),
        visible: format!("{}", item.visible),
        maximum_text: format!("{:.2}", item.maximum),
        port: item.port.to_string(),
        retries_text: item.retries.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_reservation_views(items: &[PolicyReservation]) -> Vec<PolicyReservationView> {
    let mut views: Vec<PolicyReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_reservation_view(item));
    }
    views
}

pub fn sum_policy_reservation_view(view: &PolicyReservationView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_minimum + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_reservation("code-713", "title-242");
        let view = to_policy_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
