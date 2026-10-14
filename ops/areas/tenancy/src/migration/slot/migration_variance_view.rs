use super::*;

use acme_kit::text;

/// How screens and exports show a migration variance.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationVarianceView {
    pub code: String,
    pub name_text: String,
    pub quota_units_text: String,
    pub reviewer: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_migration_variance_view(item: &MigrationVariance) -> MigrationVarianceView {
    MigrationVarianceView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        quota_units_text: item.quota_units.to_string(),
        reviewer: text::truncate(&item.reviewer, 120),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_variance_views(items: &[MigrationVariance]) -> Vec<MigrationVarianceView> {
    let mut views: Vec<MigrationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_variance_view(item));
    }
    views
}

pub fn sum_migration_variance_view(view: &MigrationVarianceView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_sequence as f64))
}
