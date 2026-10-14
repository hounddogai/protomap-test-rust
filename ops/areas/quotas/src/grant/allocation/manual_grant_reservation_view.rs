use super::*;

use acme_kit::text;

/// How screens and exports show a manual grant reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualGrantReservationView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub host: String,
    pub channel_text: String,
    pub ceiling: String,
    pub revision: String,
    pub attempts_text: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_manual_grant_reservation_view(item: &ManualGrantReservation) -> ManualGrantReservationView {
    ManualGrantReservationView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        cost: format!("{:.2}", item.cost),
        host: text::truncate(&item.host, 120),
        channel_text: item.channel.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        revision: item.revision.to_string(),
        attempts_text: item.attempts.to_string(),
        fingerprint_text: text::truncate(&item.fingerprint, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_grant_reservation_views(items: &[ManualGrantReservation]) -> Vec<ManualGrantReservationView> {
    let mut views: Vec<ManualGrantReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_grant_reservation_view(item));
    }
    views
}

pub fn sum_manual_grant_reservation_view(view: &ManualGrantReservationView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_cost + parsed_ceiling + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_grant_reservation("code-913", "label-622");
        let view = to_manual_grant_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_grant_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
