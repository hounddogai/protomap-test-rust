use super::*;

/// How screens and exports show a data export record.
#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRecordView {
    pub code: String,
    pub name: String,
    pub attempts: String,
    pub grace_days: String,
    pub quota_units: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_data_export_record_view(item: &DataExportRecord) -> DataExportRecordView {
    DataExportRecordView {
        code: item.code.clone(),
        name: item.name.clone(),
        attempts: item.attempts.to_string(),
        grace_days: item.grace_days.to_string(),
        quota_units: item.quota_units.to_string(),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_data_export_record_views(items: &[DataExportRecord]) -> Vec<DataExportRecordView> {
    let mut views: Vec<DataExportRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_data_export_record_view(item));
    }
    views
}

pub fn sum_data_export_record_view(view: &DataExportRecordView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_grace_days as f64) + (parsed_quota_units as f64))
}
