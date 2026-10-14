use super::*;

use acme_kit::text;

/// How screens and exports show a lenient zone metric.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientZoneMetricView {
    pub code: String,
    pub label: String,
    pub sequence_text: String,
    pub notes_text: String,
    pub checksum: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_lenient_zone_metric_view(item: &LenientZoneMetric) -> LenientZoneMetricView {
    LenientZoneMetricView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 120),
        sequence_text: item.sequence.to_string(),
        notes_text: item.notes.clone(),
        checksum: text::truncate(&item.checksum, 120),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_zone_metric_views(items: &[LenientZoneMetric]) -> Vec<LenientZoneMetricView> {
    let mut views: Vec<LenientZoneMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_zone_metric_view(item));
    }
    views
}

pub fn sum_lenient_zone_metric_view(view: &LenientZoneMetricView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_updated_at as f64))
}
