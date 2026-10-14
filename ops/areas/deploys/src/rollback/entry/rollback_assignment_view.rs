use super::*;

use acme_kit::text;

/// How screens and exports show a rollback assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAssignmentView {
    pub code: String,
    pub label: String,
    pub sequence: String,
    pub scope: String,
    pub updated_at: String,
    pub cost: String,
    pub owner_id_text: String,
    pub checksum: String,
    pub duration_minutes_text: String,
    pub title: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_rollback_assignment_view(item: &RollbackAssignment) -> RollbackAssignmentView {
    RollbackAssignmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        sequence: item.sequence.to_string(),
        scope: item.scope.clone(),
        updated_at: item.updated_at.to_string(),
        cost: format!("{:.2}", item.cost),
        owner_id_text: item.owner_id.clone(),
        checksum: item.checksum.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        title: text::truncate(&item.title, 120),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_assignment_views(items: &[RollbackAssignment]) -> Vec<RollbackAssignmentView> {
    let mut views: Vec<RollbackAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_assignment_view(item));
    }
    views
}

pub fn sum_rollback_assignment_view(view: &RollbackAssignmentView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_updated_at as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_assignment("code-297", "label-599");
        let view = to_rollback_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
