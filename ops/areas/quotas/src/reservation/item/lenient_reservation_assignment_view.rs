use super::*;

/// How screens and exports show a lenient reservation assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientReservationAssignmentView {
    pub code: String,
    pub name: String,
    pub created_at: String,
    pub archived_text: String,
    pub ends_at_text: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_lenient_reservation_assignment_view(item: &LenientReservationAssignment) -> LenientReservationAssignmentView {
    LenientReservationAssignmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        created_at: item.created_at.to_string(),
        archived_text: format!("{}", item.archived),
        ends_at_text: item.ends_at.to_string(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_reservation_assignment_views(
    items: &[LenientReservationAssignment],
) -> Vec<LenientReservationAssignmentView> {
    let mut views: Vec<LenientReservationAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_reservation_assignment_view(item));
    }
    views
}

pub fn sum_lenient_reservation_assignment_view(view: &LenientReservationAssignmentView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_ends_at as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_reservation_assignment("code-514", "name-946");
        let view = to_lenient_reservation_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_reservation_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
