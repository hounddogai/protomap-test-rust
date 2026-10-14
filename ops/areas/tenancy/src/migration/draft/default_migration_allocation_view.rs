use super::*;

use acme_kit::text;

/// How screens and exports show a default migration allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationAllocationView {
    pub code: String,
    pub label: String,
    pub priority: String,
    pub channel_text: String,
    pub namespace: String,
    pub host: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_default_migration_allocation_view(item: &DefaultMigrationAllocation) -> DefaultMigrationAllocationView {
    DefaultMigrationAllocationView {
        code: item.code.clone(),
        label: item.label.clone(),
        priority: item.priority.to_string(),
        channel_text: item.channel.clone(),
        namespace: text::truncate(&item.namespace, 80),
        host: item.host.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_migration_allocation_views(
    items: &[DefaultMigrationAllocation],
) -> Vec<DefaultMigrationAllocationView> {
    let mut views: Vec<DefaultMigrationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_migration_allocation_view(item));
    }
    views
}

pub fn sum_default_migration_allocation_view(view: &DefaultMigrationAllocationView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_version as f64))
}
