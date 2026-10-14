use super::*;

/// How screens and exports show a pending slo assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloAssignmentView {
    pub code_text: String,
    pub name_text: String,
    pub reviewer: String,
    pub approved_text: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_pending_slo_assignment_view(item: &PendingSloAssignment) -> PendingSloAssignmentView {
    PendingSloAssignmentView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        reviewer: item.reviewer.clone(),
        approved_text: format!("{}", item.approved),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_slo_assignment_views(items: &[PendingSloAssignment]) -> Vec<PendingSloAssignmentView> {
    let mut views: Vec<PendingSloAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_slo_assignment_view(item));
    }
    views
}

pub fn sum_pending_slo_assignment_view(view: &PendingSloAssignmentView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_slo_assignment("code-445", "name-865");
        let view = to_pending_slo_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_slo_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
