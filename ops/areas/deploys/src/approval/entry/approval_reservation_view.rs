use super::*;

use acme_kit::text;

/// How screens and exports show a approval reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalReservationView {
    pub code: String,
    pub title_text: String,
    pub ends_at: String,
    pub starts_at: String,
    pub ttl_seconds: String,
    pub expires_at_text: String,
    pub audience_text: String,
    pub status_label: String,
}

pub fn to_approval_reservation_view(item: &ApprovalReservation) -> ApprovalReservationView {
    ApprovalReservationView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        starts_at: item.starts_at.to_string(),
        ttl_seconds: item.ttl_seconds.to_string(),
        expires_at_text: item.expires_at.to_string(),
        audience_text: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_reservation_views(items: &[ApprovalReservation]) -> Vec<ApprovalReservationView> {
    let mut views: Vec<ApprovalReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_reservation_view(item));
    }
    views
}

pub fn sum_approval_reservation_view(view: &ApprovalReservationView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_starts_at as f64) + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_reservation("code-294", "title-577");
        let view = to_approval_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
