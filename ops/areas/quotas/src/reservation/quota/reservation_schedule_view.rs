use super::*;

/// How screens and exports show a reservation schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationScheduleView {
    pub code: String,
    pub title: String,
    pub description_text: String,
    pub created_at: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_reservation_schedule_view(item: &ReservationSchedule) -> ReservationScheduleView {
    ReservationScheduleView {
        code: item.code.clone(),
        title: item.title.clone(),
        description_text: item.description.clone(),
        created_at: item.created_at.to_string(),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_schedule_views(items: &[ReservationSchedule]) -> Vec<ReservationScheduleView> {
    let mut views: Vec<ReservationScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_schedule_view(item));
    }
    views
}

pub fn sum_reservation_schedule_view(view: &ReservationScheduleView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_schedule("code-236", "title-497");
        let view = to_reservation_schedule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_schedule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
