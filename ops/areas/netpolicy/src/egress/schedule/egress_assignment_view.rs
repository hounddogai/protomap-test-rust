use super::*;

use acme_kit::text;

/// How screens and exports show a egress assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressAssignmentView {
    pub code: String,
    pub title_text: String,
    pub reviewer: String,
    pub contact_email_text: String,
    pub priority: String,
    pub cluster: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_egress_assignment_view(item: &EgressAssignment) -> EgressAssignmentView {
    EgressAssignmentView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        reviewer: item.reviewer.clone(),
        contact_email_text: item.contact_email.clone(),
        priority: item.priority.to_string(),
        cluster: text::truncate(&item.cluster, 120),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_assignment_views(items: &[EgressAssignment]) -> Vec<EgressAssignmentView> {
    let mut views: Vec<EgressAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_assignment_view(item));
    }
    views
}

pub fn sum_egress_assignment_view(view: &EgressAssignmentView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_egress_assignment("code-127", "title-148");
        let view = to_egress_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_egress_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
