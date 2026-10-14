use super::*;

use acme_kit::text;

/// How screens and exports show a suspension assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAssignmentView {
    pub code: String,
    pub name_text: String,
    pub capacity: String,
    pub unit: String,
    pub reviewer_text: String,
    pub summary: String,
    pub sequence: String,
    pub key_id_text: String,
    pub timezone_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_suspension_assignment_view(item: &SuspensionAssignment) -> SuspensionAssignmentView {
    SuspensionAssignmentView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        capacity: item.capacity.to_string(),
        unit: item.unit.clone(),
        reviewer_text: text::truncate(&item.reviewer, 64),
        summary: item.summary.clone(),
        sequence: item.sequence.to_string(),
        key_id_text: text::truncate(&item.key_id, 40),
        timezone_text: text::truncate(&item.timezone, 64),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_assignment_views(items: &[SuspensionAssignment]) -> Vec<SuspensionAssignmentView> {
    let mut views: Vec<SuspensionAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_assignment_view(item));
    }
    views
}

pub fn sum_suspension_assignment_view(view: &SuspensionAssignmentView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_assignment("code-43", "name-274");
        let view = to_suspension_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
