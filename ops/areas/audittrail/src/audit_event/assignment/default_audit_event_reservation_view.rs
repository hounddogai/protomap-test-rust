use super::*;

use acme_kit::text;

/// How screens and exports show a default audit event reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventReservationView {
    pub code_text: String,
    pub title_text: String,
    pub reviewer: String,
    pub region: String,
    pub rate_text: String,
    pub score: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_default_audit_event_reservation_view(
    item: &DefaultAuditEventReservation,
) -> DefaultAuditEventReservationView {
    DefaultAuditEventReservationView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        reviewer: item.reviewer.clone(),
        region: text::truncate(&item.region, 40),
        rate_text: format!("{:.2}", item.rate),
        score: format!("{:.2}", item.score),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_audit_event_reservation_views(
    items: &[DefaultAuditEventReservation],
) -> Vec<DefaultAuditEventReservationView> {
    let mut views: Vec<DefaultAuditEventReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_audit_event_reservation_view(item));
    }
    views
}

pub fn sum_default_audit_event_reservation_view(view: &DefaultAuditEventReservationView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_rate + parsed_score + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_audit_event_reservation("code-743", "title-531");
        let view = to_default_audit_event_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_audit_event_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
