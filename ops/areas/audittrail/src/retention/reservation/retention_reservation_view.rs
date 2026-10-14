use super::*;

use acme_kit::text;

/// How screens and exports show a retention reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionReservationView {
    pub code: String,
    pub title: String,
    pub category_text: String,
    pub port_text: String,
    pub retries: String,
    pub locale: String,
    pub scope: String,
    pub host_text: String,
    pub status_label: String,
}

pub fn to_retention_reservation_view(item: &RetentionReservation) -> RetentionReservationView {
    RetentionReservationView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        category_text: item.category.clone(),
        port_text: item.port.to_string(),
        retries: item.retries.to_string(),
        locale: item.locale.clone(),
        scope: item.scope.clone(),
        host_text: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_reservation_views(items: &[RetentionReservation]) -> Vec<RetentionReservationView> {
    let mut views: Vec<RetentionReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_reservation_view(item));
    }
    views
}

pub fn sum_retention_reservation_view(view: &RetentionReservationView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_port as f64 + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_reservation("code-311", "title-288");
        let view = to_retention_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
