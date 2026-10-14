use super::*;

use acme_kit::text;

/// How screens and exports show a legacy migration checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationCheckpointView {
    pub code: String,
    pub label_text: String,
    pub audience: String,
    pub window_days_text: String,
    pub scope_text: String,
    pub timezone_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_legacy_migration_checkpoint_view(item: &LegacyMigrationCheckpoint) -> LegacyMigrationCheckpointView {
    LegacyMigrationCheckpointView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        audience: item.audience.clone(),
        window_days_text: item.window_days.to_string(),
        scope_text: text::truncate(&item.scope, 120),
        timezone_text: text::truncate(&item.timezone, 64),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_migration_checkpoint_views(items: &[LegacyMigrationCheckpoint]) -> Vec<LegacyMigrationCheckpointView> {
    let mut views: Vec<LegacyMigrationCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_migration_checkpoint_view(item));
    }
    views
}

pub fn sum_legacy_migration_checkpoint_view(view: &LegacyMigrationCheckpointView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_ceiling)
}
