use super::*;

use acme_kit::text;

/// How screens and exports show a default runbook assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRunbookAssignmentView {
    pub code: String,
    pub name: String,
    pub ttl_seconds: String,
    pub capacity_text: String,
    pub notes_text: String,
    pub segment: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_default_runbook_assignment_view(item: &DefaultRunbookAssignment) -> DefaultRunbookAssignmentView {
    DefaultRunbookAssignmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        capacity_text: item.capacity.to_string(),
        notes_text: text::truncate(&item.notes, 120),
        segment: text::truncate(&item.segment, 80),
        external_ref: item.external_ref.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_runbook_assignment_views(items: &[DefaultRunbookAssignment]) -> Vec<DefaultRunbookAssignmentView> {
    let mut views: Vec<DefaultRunbookAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_runbook_assignment_view(item));
    }
    views
}

pub fn sum_default_runbook_assignment_view(view: &DefaultRunbookAssignmentView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_runbook_assignment("code-646", "name-962");
        let view = to_default_runbook_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_runbook_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
