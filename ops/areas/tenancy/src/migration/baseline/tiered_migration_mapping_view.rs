use super::*;

use acme_kit::text;

/// How screens and exports show a tiered migration mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationMappingView {
    pub code: String,
    pub title: String,
    pub quota_units_text: String,
    pub threshold: String,
    pub minimum: String,
    pub tenant_id: String,
    pub notes: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_tiered_migration_mapping_view(item: &TieredMigrationMapping) -> TieredMigrationMappingView {
    TieredMigrationMappingView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        quota_units_text: item.quota_units.to_string(),
        threshold: format!("{:.2}", item.threshold),
        minimum: format!("{:.2}", item.minimum),
        tenant_id: item.tenant_id.clone(),
        notes: item.notes.clone(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_migration_mapping_views(items: &[TieredMigrationMapping]) -> Vec<TieredMigrationMappingView> {
    let mut views: Vec<TieredMigrationMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_migration_mapping_view(item));
    }
    views
}

pub fn sum_tiered_migration_mapping_view(view: &TieredMigrationMappingView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_threshold + parsed_minimum)
}
