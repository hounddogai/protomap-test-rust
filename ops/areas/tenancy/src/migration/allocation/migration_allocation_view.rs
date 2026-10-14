use super::*;

use acme_kit::text;

/// How screens and exports show a migration allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAllocationView {
    pub code_text: String,
    pub label_text: String,
    pub budget: String,
    pub ratio: String,
    pub sequence: String,
    pub timezone: String,
    pub title: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_migration_allocation_view(item: &MigrationAllocation) -> MigrationAllocationView {
    MigrationAllocationView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        budget: format!("{:.2}", item.budget),
        ratio: format!("{:.2}", item.ratio),
        sequence: item.sequence.to_string(),
        timezone: item.timezone.clone(),
        title: text::truncate(&item.title, 80),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_allocation_views(items: &[MigrationAllocation]) -> Vec<MigrationAllocationView> {
    let mut views: Vec<MigrationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_allocation_view(item));
    }
    views
}

pub fn sum_migration_allocation_view(view: &MigrationAllocationView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_budget + parsed_ratio + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_allocation("code-371", "label-780");
        let view = to_migration_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
