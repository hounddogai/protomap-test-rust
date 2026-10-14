use super::*;

use acme_kit::text;

/// How screens and exports show a capped migration cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationCycleView {
    pub code_text: String,
    pub name: String,
    pub source: String,
    pub ends_at: String,
    pub status_label: String,
}

pub fn to_capped_migration_cycle_view(item: &CappedMigrationCycle) -> CappedMigrationCycleView {
    CappedMigrationCycleView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 120),
        source: item.source.clone(),
        ends_at: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_migration_cycle_views(items: &[CappedMigrationCycle]) -> Vec<CappedMigrationCycleView> {
    let mut views: Vec<CappedMigrationCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_migration_cycle_view(item));
    }
    views
}

pub fn sum_capped_migration_cycle_view(view: &CappedMigrationCycleView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_migration_cycle("code-551", "name-566");
        let view = to_capped_migration_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_migration_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
