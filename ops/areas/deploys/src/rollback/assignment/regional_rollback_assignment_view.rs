use super::*;

use acme_kit::text;

/// How screens and exports show a regional rollback assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalRollbackAssignmentView {
    pub code: String,
    pub label_text: String,
    pub duration_minutes: String,
    pub contact_email_text: String,
    pub discount: String,
    pub unit: String,
    pub reviewer_text: String,
    pub batch_size_text: String,
    pub rate: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_regional_rollback_assignment_view(item: &RegionalRollbackAssignment) -> RegionalRollbackAssignmentView {
    RegionalRollbackAssignmentView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        duration_minutes: item.duration_minutes.to_string(),
        contact_email_text: text::truncate(&item.contact_email, 80),
        discount: format!("{:.2}", item.discount),
        unit: item.unit.clone(),
        reviewer_text: item.reviewer.clone(),
        batch_size_text: item.batch_size.to_string(),
        rate: format!("{:.2}", item.rate),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_rollback_assignment_views(
    items: &[RegionalRollbackAssignment],
) -> Vec<RegionalRollbackAssignmentView> {
    let mut views: Vec<RegionalRollbackAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_rollback_assignment_view(item));
    }
    views
}

pub fn sum_regional_rollback_assignment_view(view: &RegionalRollbackAssignmentView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_discount + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_rollback_assignment("code-307", "label-513");
        let view = to_regional_rollback_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_rollback_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
