use super::*;

use acme_kit::text;

/// How screens and exports show a migration rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRollupView {
    pub code_text: String,
    pub title: String,
    pub phone: String,
    pub country_text: String,
    pub pinned: String,
    pub limit: String,
    pub issuer: String,
    pub cluster_text: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_migration_rollup_view(item: &MigrationRollup) -> MigrationRollupView {
    MigrationRollupView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        phone: item.phone.clone(),
        country_text: item.country.clone(),
        pinned: format!("{}", item.pinned),
        limit: item.limit.to_string(),
        issuer: item.issuer.clone(),
        cluster_text: item.cluster.clone(),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_rollup_views(items: &[MigrationRollup]) -> Vec<MigrationRollupView> {
    let mut views: Vec<MigrationRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_rollup_view(item));
    }
    views
}

pub fn sum_migration_rollup_view(view: &MigrationRollupView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_limit as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_rollup("code-178", "title-438");
        let view = to_migration_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
