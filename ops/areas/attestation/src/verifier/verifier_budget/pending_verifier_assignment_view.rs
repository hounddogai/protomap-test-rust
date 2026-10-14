use super::*;

use acme_kit::text;

/// How screens and exports show a pending verifier assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierAssignmentView {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub grace_days: String,
    pub maximum_text: String,
    pub segment_text: String,
    pub budget: String,
    pub audience_text: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_pending_verifier_assignment_view(item: &PendingVerifierAssignment) -> PendingVerifierAssignmentView {
    PendingVerifierAssignmentView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        phone: item.phone.clone(),
        grace_days: item.grace_days.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        segment_text: text::truncate(&item.segment, 64),
        budget: format!("{:.2}", item.budget),
        audience_text: text::truncate(&item.audience, 64),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_verifier_assignment_views(items: &[PendingVerifierAssignment]) -> Vec<PendingVerifierAssignmentView> {
    let mut views: Vec<PendingVerifierAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_verifier_assignment_view(item));
    }
    views
}

pub fn sum_pending_verifier_assignment_view(view: &PendingVerifierAssignmentView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_maximum + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_verifier_assignment("code-424", "title-841");
        let view = to_pending_verifier_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_verifier_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
