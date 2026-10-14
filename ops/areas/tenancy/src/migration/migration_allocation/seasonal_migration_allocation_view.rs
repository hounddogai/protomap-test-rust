use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal migration allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationAllocationView {
    pub code: String,
    pub title: String,
    pub contact_email: String,
    pub name_text: String,
    pub timezone: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_seasonal_migration_allocation_view(item: &SeasonalMigrationAllocation) -> SeasonalMigrationAllocationView {
    SeasonalMigrationAllocationView {
        code: item.code.clone(),
        title: item.title.clone(),
        contact_email: text::truncate(&item.contact_email, 64),
        name_text: item.name.clone(),
        timezone: text::truncate(&item.timezone, 80),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_migration_allocation_views(
    items: &[SeasonalMigrationAllocation],
) -> Vec<SeasonalMigrationAllocationView> {
    let mut views: Vec<SeasonalMigrationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_migration_allocation_view(item));
    }
    views
}

pub fn sum_seasonal_migration_allocation_view(view: &SeasonalMigrationAllocationView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_migration_allocation("code-972", "title-552");
        let view = to_seasonal_migration_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_migration_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
