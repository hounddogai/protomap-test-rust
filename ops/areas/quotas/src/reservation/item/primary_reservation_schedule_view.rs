use super::*;

/// How screens and exports show a primary reservation schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationScheduleView {
    pub code_text: String,
    pub label: String,
    pub discount: String,
    pub grace_days: String,
    pub issuer_text: String,
    pub region_text: String,
    pub notes: String,
    pub window_days_text: String,
    pub pinned: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_primary_reservation_schedule_view(item: &PrimaryReservationSchedule) -> PrimaryReservationScheduleView {
    PrimaryReservationScheduleView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        grace_days: item.grace_days.to_string(),
        issuer_text: item.issuer.clone(),
        region_text: item.region.clone(),
        notes: item.notes.clone(),
        window_days_text: item.window_days.to_string(),
        pinned: format!("{}", item.pinned),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_reservation_schedule_views(
    items: &[PrimaryReservationSchedule],
) -> Vec<PrimaryReservationScheduleView> {
    let mut views: Vec<PrimaryReservationScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_reservation_schedule_view(item));
    }
    views
}

pub fn sum_primary_reservation_schedule_view(view: &PrimaryReservationScheduleView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_discount + (parsed_grace_days as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_reservation_schedule("code-397", "label-418");
        let view = to_primary_reservation_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_reservation_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
