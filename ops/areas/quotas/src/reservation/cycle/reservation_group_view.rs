use super::*;

use acme_kit::text;

/// How screens and exports show a reservation group.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationGroupView {
    pub code: String,
    pub label_text: String,
    pub burst: String,
    pub maximum: String,
    pub port: String,
    pub phone: String,
    pub updated_at_text: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_reservation_group_view(item: &ReservationGroup) -> ReservationGroupView {
    ReservationGroupView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        burst: item.burst.to_string(),
        maximum: format!("{:.2}", item.maximum),
        port: item.port.to_string(),
        phone: text::truncate(&item.phone, 120),
        updated_at_text: item.updated_at.to_string(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_group_views(items: &[ReservationGroup]) -> Vec<ReservationGroupView> {
    let mut views: Vec<ReservationGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_group_view(item));
    }
    views
}

pub fn sum_reservation_group_view(view: &ReservationGroupView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_burst as f64 + parsed_maximum + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_group("code-407", "label-297");
        let view = to_reservation_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
