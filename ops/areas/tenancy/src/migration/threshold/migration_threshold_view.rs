use super::*;

use acme_kit::text;

/// How screens and exports show a migration threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationThresholdView {
    pub code_text: String,
    pub label_text: String,
    pub capacity: String,
    pub scope_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_migration_threshold_view(item: &MigrationThreshold) -> MigrationThresholdView {
    MigrationThresholdView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        capacity: item.capacity.to_string(),
        scope_text: text::truncate(&item.scope, 40),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_threshold_views(items: &[MigrationThreshold]) -> Vec<MigrationThresholdView> {
    let mut views: Vec<MigrationThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_threshold_view(item));
    }
    views
}

pub fn sum_migration_threshold_view(view: &MigrationThresholdView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_capacity as f64)
}
