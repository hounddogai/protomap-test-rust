use super::*;

use acme_kit::text;

/// How screens and exports show a batched approval reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedApprovalReservationView {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub updated_at: String,
    pub cluster: String,
    pub scope_text: String,
    pub limit_text: String,
    pub quantity: String,
    pub checksum: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_batched_approval_reservation_view(item: &BatchedApprovalReservation) -> BatchedApprovalReservationView {
    BatchedApprovalReservationView {
        code: item.code.clone(),
        label: item.label.clone(),
        reviewer: text::truncate(&item.reviewer, 64),
        updated_at: item.updated_at.to_string(),
        cluster: item.cluster.clone(),
        scope_text: item.scope.clone(),
        limit_text: item.limit.to_string(),
        quantity: item.quantity.to_string(),
        checksum: item.checksum.clone(),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_approval_reservation_views(
    items: &[BatchedApprovalReservation],
) -> Vec<BatchedApprovalReservationView> {
    let mut views: Vec<BatchedApprovalReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_approval_reservation_view(item));
    }
    views
}

pub fn sum_batched_approval_reservation_view(view: &BatchedApprovalReservationView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_limit as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_approval_reservation("code-12", "label-835");
        let view = to_batched_approval_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_approval_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
