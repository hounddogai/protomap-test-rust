use super::*;

use acme_kit::text;

/// How screens and exports show a batched region pin reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRegionPinReservationView {
    pub code: String,
    pub name: String,
    pub active_text: String,
    pub principal_text: String,
    pub expires_at_text: String,
    pub scope: String,
    pub owner_id: String,
    pub tenant_id_text: String,
    pub priority_text: String,
    pub cipher_text: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_batched_region_pin_reservation_view(item: &BatchedRegionPinReservation) -> BatchedRegionPinReservationView {
    BatchedRegionPinReservationView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        active_text: format!("{}", item.active),
        principal_text: text::truncate(&item.principal, 80),
        expires_at_text: item.expires_at.to_string(),
        scope: item.scope.clone(),
        owner_id: item.owner_id.clone(),
        tenant_id_text: item.tenant_id.clone(),
        priority_text: item.priority.to_string(),
        cipher_text: item.cipher.clone(),
        checksum_text: text::truncate(&item.checksum, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_region_pin_reservation_views(
    items: &[BatchedRegionPinReservation],
) -> Vec<BatchedRegionPinReservationView> {
    let mut views: Vec<BatchedRegionPinReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_region_pin_reservation_view(item));
    }
    views
}

pub fn sum_batched_region_pin_reservation_view(view: &BatchedRegionPinReservationView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_region_pin_reservation("code-551", "name-23");
        let view = to_batched_region_pin_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_region_pin_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
