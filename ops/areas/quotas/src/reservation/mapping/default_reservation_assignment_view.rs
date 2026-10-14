use super::*;

use acme_kit::text;

/// How screens and exports show a default reservation assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationAssignmentView {
    pub code: String,
    pub name: String,
    pub rate_text: String,
    pub archived: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_default_reservation_assignment_view(item: &DefaultReservationAssignment) -> DefaultReservationAssignmentView {
    DefaultReservationAssignmentView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        rate_text: format!("{:.2}", item.rate),
        archived: format!("{}", item.archived),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_reservation_assignment_views(
    items: &[DefaultReservationAssignment],
) -> Vec<DefaultReservationAssignmentView> {
    let mut views: Vec<DefaultReservationAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_reservation_assignment_view(item));
    }
    views
}

pub fn sum_default_reservation_assignment_view(view: &DefaultReservationAssignmentView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_reservation_assignment("code-321", "name-693");
        let view = to_default_reservation_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_reservation_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
