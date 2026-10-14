use super::*;

/// How screens and exports show a health check reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReservationView {
    pub code: String,
    pub name_text: String,
    pub summary_text: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_health_check_reservation_view(item: &HealthCheckReservation) -> HealthCheckReservationView {
    HealthCheckReservationView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        summary_text: item.summary.clone(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_reservation_views(items: &[HealthCheckReservation]) -> Vec<HealthCheckReservationView> {
    let mut views: Vec<HealthCheckReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_reservation_view(item));
    }
    views
}

pub fn sum_health_check_reservation_view(view: &HealthCheckReservationView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_capacity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_health_check_reservation("code-491", "name-898");
        let view = to_health_check_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_health_check_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
