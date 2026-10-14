use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled zone reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledZoneReservationView {
    pub code_text: String,
    pub label: String,
    pub name: String,
    pub category_text: String,
    pub grace_days: String,
    pub fingerprint: String,
    pub timezone: String,
    pub external_ref: String,
    pub rotation_days_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_scheduled_zone_reservation_view(item: &ScheduledZoneReservation) -> ScheduledZoneReservationView {
    ScheduledZoneReservationView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        name: text::truncate(&item.name, 64),
        category_text: item.category.clone(),
        grace_days: item.grace_days.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        timezone: item.timezone.clone(),
        external_ref: text::truncate(&item.external_ref, 64),
        rotation_days_text: item.rotation_days.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_zone_reservation_views(items: &[ScheduledZoneReservation]) -> Vec<ScheduledZoneReservationView> {
    let mut views: Vec<ScheduledZoneReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_zone_reservation_view(item));
    }
    views
}

pub fn sum_scheduled_zone_reservation_view(view: &ScheduledZoneReservationView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_rotation_days as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_zone_reservation("code-223", "label-474");
        let view = to_scheduled_zone_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_zone_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
