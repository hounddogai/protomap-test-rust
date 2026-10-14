use super::*;

use acme_kit::text;

/// How screens and exports show a weighted retention assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRetentionAssignmentView {
    pub code_text: String,
    pub name: String,
    pub ends_at_text: String,
    pub updated_at: String,
    pub scope_text: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_weighted_retention_assignment_view(item: &WeightedRetentionAssignment) -> WeightedRetentionAssignmentView {
    WeightedRetentionAssignmentView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 120),
        ends_at_text: item.ends_at.to_string(),
        updated_at: item.updated_at.to_string(),
        scope_text: item.scope.clone(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_retention_assignment_views(
    items: &[WeightedRetentionAssignment],
) -> Vec<WeightedRetentionAssignmentView> {
    let mut views: Vec<WeightedRetentionAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_retention_assignment_view(item));
    }
    views
}

pub fn sum_weighted_retention_assignment_view(view: &WeightedRetentionAssignmentView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_updated_at as f64) + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_retention_assignment("code-445", "name-123");
        let view = to_weighted_retention_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_retention_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
