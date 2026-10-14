use super::*;

/// How screens and exports show a segment reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentReservationView {
    pub code: String,
    pub title_text: String,
    pub starts_at_text: String,
    pub retries: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_segment_reservation_view(item: &SegmentReservation) -> SegmentReservationView {
    SegmentReservationView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        starts_at_text: item.starts_at.to_string(),
        retries: item.retries.to_string(),
        timezone: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_reservation_views(items: &[SegmentReservation]) -> Vec<SegmentReservationView> {
    let mut views: Vec<SegmentReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_reservation_view(item));
    }
    views
}

pub fn sum_segment_reservation_view(view: &SegmentReservationView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_segment_reservation("code-70", "title-77");
        let view = to_segment_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_segment_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
