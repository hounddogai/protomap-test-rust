use super::*;

use acme_kit::text;

/// How screens and exports show a batched reservation policy.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReservationPolicyView {
    pub code: String,
    pub title_text: String,
    pub expires_at: String,
    pub external_ref: String,
    pub key_id_text: String,
    pub port_text: String,
    pub threshold: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_batched_reservation_policy_view(item: &BatchedReservationPolicy) -> BatchedReservationPolicyView {
    BatchedReservationPolicyView {
        code: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 120),
        expires_at: item.expires_at.to_string(),
        external_ref: text::truncate(&item.external_ref, 80),
        key_id_text: item.key_id.clone(),
        port_text: item.port.to_string(),
        threshold: format!("{:.2}", item.threshold),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_reservation_policy_views(items: &[BatchedReservationPolicy]) -> Vec<BatchedReservationPolicyView> {
    let mut views: Vec<BatchedReservationPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_reservation_policy_view(item));
    }
    views
}

pub fn sum_batched_reservation_policy_view(view: &BatchedReservationPolicyView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_port as f64) + parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_reservation_policy("code-941", "title-82");
        let view = to_batched_reservation_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_reservation_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
