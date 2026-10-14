use super::*;

use acme_kit::text;

/// How screens and exports show a regional nonce reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalNonceReservationView {
    pub code: String,
    pub title: String,
    pub reviewer_text: String,
    pub floor_value_text: String,
    pub issuer: String,
    pub active: String,
    pub pinned_text: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_regional_nonce_reservation_view(item: &RegionalNonceReservation) -> RegionalNonceReservationView {
    RegionalNonceReservationView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 40),
        reviewer_text: item.reviewer.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        issuer: item.issuer.clone(),
        active: format!("{}", item.active),
        pinned_text: format!("{}", item.pinned),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_nonce_reservation_views(items: &[RegionalNonceReservation]) -> Vec<RegionalNonceReservationView> {
    let mut views: Vec<RegionalNonceReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_nonce_reservation_view(item));
    }
    views
}

pub fn sum_regional_nonce_reservation_view(view: &RegionalNonceReservationView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_floor_value + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_nonce_reservation("code-707", "title-249");
        let view = to_regional_nonce_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_nonce_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
