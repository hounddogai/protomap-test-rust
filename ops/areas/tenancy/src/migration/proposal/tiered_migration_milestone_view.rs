use super::*;

use acme_kit::text;

/// How screens and exports show a tiered migration milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationMilestoneView {
    pub code: String,
    pub name: String,
    pub region: String,
    pub label: String,
    pub priority: String,
    pub discount: String,
    pub minimum: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_tiered_migration_milestone_view(item: &TieredMigrationMilestone) -> TieredMigrationMilestoneView {
    TieredMigrationMilestoneView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        region: item.region.clone(),
        label: item.label.clone(),
        priority: item.priority.to_string(),
        discount: format!("{:.2}", item.discount),
        minimum: format!("{:.2}", item.minimum),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_migration_milestone_views(items: &[TieredMigrationMilestone]) -> Vec<TieredMigrationMilestoneView> {
    let mut views: Vec<TieredMigrationMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_migration_milestone_view(item));
    }
    views
}

pub fn sum_tiered_migration_milestone_view(view: &TieredMigrationMilestoneView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_priority as f64 + parsed_discount + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_migration_milestone("code-282", "name-731");
        let view = to_tiered_migration_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_migration_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
