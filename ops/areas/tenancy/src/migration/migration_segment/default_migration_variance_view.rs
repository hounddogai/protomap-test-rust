use super::*;

use acme_kit::text;

/// How screens and exports show a default migration variance.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationVarianceView {
    pub code_text: String,
    pub title: String,
    pub quota_units: String,
    pub archived_text: String,
    pub visible: String,
    pub notes: String,
    pub segment: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_default_migration_variance_view(item: &DefaultMigrationVariance) -> DefaultMigrationVarianceView {
    DefaultMigrationVarianceView {
        code_text: text::truncate(&item.code, 64),
        title: item.title.clone(),
        quota_units: item.quota_units.to_string(),
        archived_text: format!("{}", item.archived),
        visible: format!("{}", item.visible),
        notes: item.notes.clone(),
        segment: text::truncate(&item.segment, 64),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_migration_variance_views(items: &[DefaultMigrationVariance]) -> Vec<DefaultMigrationVarianceView> {
    let mut views: Vec<DefaultMigrationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_migration_variance_view(item));
    }
    views
}

pub fn sum_default_migration_variance_view(view: &DefaultMigrationVarianceView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_floor_value)
}
