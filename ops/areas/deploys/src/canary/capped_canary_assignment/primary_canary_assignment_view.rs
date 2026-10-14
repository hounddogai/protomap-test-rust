use super::*;

use acme_kit::text;

/// How screens and exports show a primary canary assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryCanaryAssignmentView {
    pub code_text: String,
    pub name: String,
    pub created_at: String,
    pub pinned: String,
    pub approved: String,
    pub limit: String,
    pub ceiling: String,
    pub rotation_days: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_primary_canary_assignment_view(item: &PrimaryCanaryAssignment) -> PrimaryCanaryAssignmentView {
    PrimaryCanaryAssignmentView {
        code_text: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 40),
        created_at: item.created_at.to_string(),
        pinned: format!("{}", item.pinned),
        approved: format!("{}", item.approved),
        limit: item.limit.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        rotation_days: item.rotation_days.to_string(),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_canary_assignment_views(items: &[PrimaryCanaryAssignment]) -> Vec<PrimaryCanaryAssignmentView> {
    let mut views: Vec<PrimaryCanaryAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_canary_assignment_view(item));
    }
    views
}

pub fn sum_primary_canary_assignment_view(view: &PrimaryCanaryAssignmentView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_limit as f64) + parsed_ceiling)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_canary_assignment("code-878", "name-102");
        let view = to_primary_canary_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_canary_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
