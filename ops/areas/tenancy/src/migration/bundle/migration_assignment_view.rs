use super::*;

use acme_kit::text;

/// How screens and exports show a migration assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAssignmentView {
    pub code_text: String,
    pub title: String,
    pub namespace: String,
    pub segment_text: String,
    pub issuer: String,
    pub currency_text: String,
    pub weight: String,
    pub approved: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_migration_assignment_view(item: &MigrationAssignment) -> MigrationAssignmentView {
    MigrationAssignmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        namespace: item.namespace.clone(),
        segment_text: item.segment.clone(),
        issuer: item.issuer.clone(),
        currency_text: text::truncate(&item.currency, 120),
        weight: format!("{:.2}", item.weight),
        approved: format!("{}", item.approved),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_assignment_views(items: &[MigrationAssignment]) -> Vec<MigrationAssignmentView> {
    let mut views: Vec<MigrationAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_assignment_view(item));
    }
    views
}

pub fn sum_migration_assignment_view(view: &MigrationAssignmentView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_weight + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_assignment("code-374", "title-752");
        let view = to_migration_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
