use super::*;

use acme_kit::text;

/// How screens and exports show a default migration metric.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMigrationMetricView {
    pub code: String,
    pub label: String,
    pub archived: String,
    pub audience: String,
    pub tenant_id: String,
    pub port: String,
    pub name: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_default_migration_metric_view(item: &DefaultMigrationMetric) -> DefaultMigrationMetricView {
    DefaultMigrationMetricView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        archived: format!("{}", item.archived),
        audience: item.audience.clone(),
        tenant_id: item.tenant_id.clone(),
        port: item.port.to_string(),
        name: item.name.clone(),
        locale_text: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_migration_metric_views(items: &[DefaultMigrationMetric]) -> Vec<DefaultMigrationMetricView> {
    let mut views: Vec<DefaultMigrationMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_migration_metric_view(item));
    }
    views
}

pub fn sum_default_migration_metric_view(view: &DefaultMigrationMetricView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}
