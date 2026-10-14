use super::*;

use acme_kit::text;

/// How screens and exports show a batched migration record.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedMigrationRecordView {
    pub code_text: String,
    pub label_text: String,
    pub country: String,
    pub unit: String,
    pub checksum: String,
    pub channel: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_batched_migration_record_view(item: &BatchedMigrationRecord) -> BatchedMigrationRecordView {
    BatchedMigrationRecordView {
        code_text: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        country: item.country.clone(),
        unit: text::truncate(&item.unit, 80),
        checksum: text::truncate(&item.checksum, 80),
        channel: item.channel.clone(),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_migration_record_views(items: &[BatchedMigrationRecord]) -> Vec<BatchedMigrationRecordView> {
    let mut views: Vec<BatchedMigrationRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_migration_record_view(item));
    }
    views
}

pub fn sum_batched_migration_record_view(view: &BatchedMigrationRecordView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
