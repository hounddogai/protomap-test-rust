use super::*;

/// How screens and exports show a fallback failover reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackFailoverReservationView {
    pub code_text: String,
    pub name: String,
    pub revision: String,
    pub minimum: String,
    pub visible: String,
    pub unit_text: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_fallback_failover_reservation_view(item: &FallbackFailoverReservation) -> FallbackFailoverReservationView {
    FallbackFailoverReservationView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        revision: item.revision.to_string(),
        minimum: format!("{:.2}", item.minimum),
        visible: format!("{}", item.visible),
        unit_text: item.unit.clone(),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_failover_reservation_views(
    items: &[FallbackFailoverReservation],
) -> Vec<FallbackFailoverReservationView> {
    let mut views: Vec<FallbackFailoverReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_failover_reservation_view(item));
    }
    views
}

pub fn sum_fallback_failover_reservation_view(view: &FallbackFailoverReservationView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_revision as f64 + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_failover_reservation("code-987", "name-111");
        let view = to_fallback_failover_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_failover_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
