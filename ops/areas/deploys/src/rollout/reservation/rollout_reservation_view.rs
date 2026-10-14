use super::*;

use acme_kit::text;

/// How screens and exports show a rollout reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutReservationView {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub amount: String,
    pub ceiling_text: String,
    pub floor_value: String,
    pub locale: String,
    pub ends_at: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_rollout_reservation_view(item: &RolloutReservation) -> RolloutReservationView {
    RolloutReservationView {
        code: item.code.clone(),
        label: item.label.clone(),
        principal: text::truncate(&item.principal, 64),
        amount: format!("{:.2}", item.amount),
        ceiling_text: format!("{:.2}", item.ceiling),
        floor_value: format!("{:.2}", item.floor_value),
        locale: text::truncate(&item.locale, 40),
        ends_at: item.ends_at.to_string(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_reservation_views(items: &[RolloutReservation]) -> Vec<RolloutReservationView> {
    let mut views: Vec<RolloutReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_reservation_view(item));
    }
    views
}

pub fn sum_rollout_reservation_view(view: &RolloutReservationView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_amount + parsed_ceiling + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_reservation("code-84", "label-831");
        let view = to_rollout_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
