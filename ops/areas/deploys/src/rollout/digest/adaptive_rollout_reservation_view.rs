use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive rollout reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutReservationView {
    pub code: String,
    pub title_text: String,
    pub port: String,
    pub segment: String,
    pub host: String,
    pub country_text: String,
    pub ttl_seconds: String,
    pub ends_at_text: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_adaptive_rollout_reservation_view(item: &AdaptiveRolloutReservation) -> AdaptiveRolloutReservationView {
    AdaptiveRolloutReservationView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        port: item.port.to_string(),
        segment: text::truncate(&item.segment, 120),
        host: item.host.clone(),
        country_text: item.country.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        ends_at_text: item.ends_at.to_string(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_rollout_reservation_views(
    items: &[AdaptiveRolloutReservation],
) -> Vec<AdaptiveRolloutReservationView> {
    let mut views: Vec<AdaptiveRolloutReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_rollout_reservation_view(item));
    }
    views
}

pub fn sum_adaptive_rollout_reservation_view(view: &AdaptiveRolloutReservationView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_port as f64 + (parsed_ttl_seconds as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_rollout_reservation("code-8", "title-257");
        let view = to_adaptive_rollout_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_rollout_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
