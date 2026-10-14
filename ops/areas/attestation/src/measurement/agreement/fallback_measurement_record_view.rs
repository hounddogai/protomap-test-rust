use super::*;

use acme_kit::text;

/// How screens and exports show a fallback measurement record.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMeasurementRecordView {
    pub code_text: String,
    pub title_text: String,
    pub sequence: String,
    pub segment: String,
    pub unit_text: String,
    pub ratio_text: String,
    pub archived: String,
    pub starts_at_text: String,
    pub currency_text: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_fallback_measurement_record_view(item: &FallbackMeasurementRecord) -> FallbackMeasurementRecordView {
    FallbackMeasurementRecordView {
        code_text: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        sequence: item.sequence.to_string(),
        segment: item.segment.clone(),
        unit_text: item.unit.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        archived: format!("{}", item.archived),
        starts_at_text: item.starts_at.to_string(),
        currency_text: item.currency.clone(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_measurement_record_views(items: &[FallbackMeasurementRecord]) -> Vec<FallbackMeasurementRecordView> {
    let mut views: Vec<FallbackMeasurementRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_measurement_record_view(item));
    }
    views
}

pub fn sum_fallback_measurement_record_view(view: &FallbackMeasurementRecordView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_ratio + (parsed_starts_at as f64))
}
