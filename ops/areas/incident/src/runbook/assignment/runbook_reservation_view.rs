use super::*;

use acme_kit::text;

/// How screens and exports show a runbook reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReservationView {
    pub code: String,
    pub name: String,
    pub retries: String,
    pub weight: String,
    pub title: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_runbook_reservation_view(item: &RunbookReservation) -> RunbookReservationView {
    RunbookReservationView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 64),
        retries: item.retries.to_string(),
        weight: format!("{:.2}", item.weight),
        title: text::truncate(&item.title, 40),
        contact_email: text::truncate(&item.contact_email, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_reservation_views(items: &[RunbookReservation]) -> Vec<RunbookReservationView> {
    let mut views: Vec<RunbookReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_reservation_view(item));
    }
    views
}

pub fn sum_runbook_reservation_view(view: &RunbookReservationView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_retries as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_reservation("code-465", "name-62");
        let view = to_runbook_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
