use super::*;

use acme_kit::text;

/// How screens and exports show a manual policy assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPolicyAssignmentView {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub source: String,
    pub notes: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_manual_policy_assignment_view(item: &ManualPolicyAssignment) -> ManualPolicyAssignmentView {
    ManualPolicyAssignmentView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        owner_id: item.owner_id.clone(),
        source: item.source.clone(),
        notes: item.notes.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_policy_assignment_views(items: &[ManualPolicyAssignment]) -> Vec<ManualPolicyAssignmentView> {
    let mut views: Vec<ManualPolicyAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_policy_assignment_view(item));
    }
    views
}

pub fn sum_manual_policy_assignment_view(view: &ManualPolicyAssignmentView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_policy_assignment("code-79", "name-550");
        let view = to_manual_policy_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_policy_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
