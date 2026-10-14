use super::*;

use acme_kit::text;

/// How screens and exports show a capped migration batch.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationBatchView {
    pub code_text: String,
    pub name_text: String,
    pub floor_value_text: String,
    pub ratio: String,
    pub namespace: String,
    pub label: String,
    pub cluster_text: String,
    pub status_label: String,
}

pub fn to_capped_migration_batch_view(item: &CappedMigrationBatch) -> CappedMigrationBatchView {
    CappedMigrationBatchView {
        code_text: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 64),
        floor_value_text: format!("{:.2}", item.floor_value),
        ratio: format!("{:.2}", item.ratio),
        namespace: item.namespace.clone(),
        label: item.label.clone(),
        cluster_text: text::truncate(&item.cluster, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_migration_batch_views(items: &[CappedMigrationBatch]) -> Vec<CappedMigrationBatchView> {
    let mut views: Vec<CappedMigrationBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_migration_batch_view(item));
    }
    views
}

pub fn sum_capped_migration_batch_view(view: &CappedMigrationBatchView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_floor_value + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_migration_batch("code-319", "name-570");
        let view = to_capped_migration_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_migration_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
