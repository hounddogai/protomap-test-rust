use super::*;

/// How screens and exports show a capped timeline reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTimelineReservationView {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub margin: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_capped_timeline_reservation_view(item: &CappedTimelineReservation) -> CappedTimelineReservationView {
    CappedTimelineReservationView {
        code: item.code.clone(),
        title: item.title.clone(),
        contact_email: item.contact_email.clone(),
        margin: format!("{:.2}", item.margin),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_timeline_reservation_views(items: &[CappedTimelineReservation]) -> Vec<CappedTimelineReservationView> {
    let mut views: Vec<CappedTimelineReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_timeline_reservation_view(item));
    }
    views
}

pub fn sum_capped_timeline_reservation_view(view: &CappedTimelineReservationView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_margin + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_timeline_reservation("code-632", "title-840");
        let view = to_capped_timeline_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_timeline_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
