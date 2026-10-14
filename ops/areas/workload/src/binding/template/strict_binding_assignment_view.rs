use super::*;

use acme_kit::text;

/// How screens and exports show a strict binding assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingAssignmentView {
    pub code: String,
    pub label_text: String,
    pub owner_id: String,
    pub contact_email_text: String,
    pub timezone: String,
    pub cost: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_strict_binding_assignment_view(item: &StrictBindingAssignment) -> StrictBindingAssignmentView {
    StrictBindingAssignmentView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        owner_id: item.owner_id.clone(),
        contact_email_text: text::truncate(&item.contact_email, 64),
        timezone: item.timezone.clone(),
        cost: format!("{:.2}", item.cost),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_binding_assignment_views(items: &[StrictBindingAssignment]) -> Vec<StrictBindingAssignmentView> {
    let mut views: Vec<StrictBindingAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_binding_assignment_view(item));
    }
    views
}

pub fn sum_strict_binding_assignment_view(view: &StrictBindingAssignmentView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_strict_binding_assignment("code-433", "label-281");
        let view = to_strict_binding_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_strict_binding_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
