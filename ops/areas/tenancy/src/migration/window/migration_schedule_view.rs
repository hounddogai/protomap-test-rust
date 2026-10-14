use super::*;

use acme_kit::text;

/// How screens and exports show a migration schedule.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScheduleView {
    pub code: String,
    pub label_text: String,
    pub title_text: String,
    pub quantity_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_migration_schedule_view(item: &MigrationSchedule) -> MigrationScheduleView {
    MigrationScheduleView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        title_text: item.title.clone(),
        quantity_text: item.quantity.to_string(),
        reviewer: text::truncate(&item.reviewer, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_schedule_views(items: &[MigrationSchedule]) -> Vec<MigrationScheduleView> {
    let mut views: Vec<MigrationScheduleView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_schedule_view(item));
    }
    views
}

pub fn sum_migration_schedule_view(view: &MigrationScheduleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
