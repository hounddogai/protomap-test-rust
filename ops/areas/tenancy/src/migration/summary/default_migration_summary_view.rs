use super::*;

use acme_kit::text;

/// How screens and exports show a default migration summary.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationSummaryView {
    pub code_text: String,
    pub name: String,
    pub segment: String,
    pub reviewer: String,
    pub audience_text: String,
    pub version_text: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_default_migration_summary_view(item: &DefaultMigrationSummary) -> DefaultMigrationSummaryView {
    DefaultMigrationSummaryView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        segment: text::truncate(&item.segment, 80),
        reviewer: item.reviewer.clone(),
        audience_text: text::truncate(&item.audience, 120),
        version_text: item.version.to_string(),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_migration_summary_views(items: &[DefaultMigrationSummary]) -> Vec<DefaultMigrationSummaryView> {
    let mut views: Vec<DefaultMigrationSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_migration_summary_view(item));
    }
    views
}

pub fn sum_default_migration_summary_view(view: &DefaultMigrationSummaryView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_version as f64 + (parsed_updated_at as f64))
}
