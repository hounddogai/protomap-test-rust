use super::*;

use acme_kit::text;

/// How screens and exports show a redaction adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAdjustmentView {
    pub code: String,
    pub label_text: String,
    pub fingerprint_text: String,
    pub quota_units: String,
    pub floor_value: String,
    pub archived: String,
    pub phone: String,
    pub weight_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_redaction_adjustment_view(item: &RedactionAdjustment) -> RedactionAdjustmentView {
    RedactionAdjustmentView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        fingerprint_text: item.fingerprint.clone(),
        quota_units: item.quota_units.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        archived: format!("{}", item.archived),
        phone: item.phone.clone(),
        weight_text: format!("{:.2}", item.weight),
        reviewer: item.reviewer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_adjustment_views(items: &[RedactionAdjustment]) -> Vec<RedactionAdjustmentView> {
    let mut views: Vec<RedactionAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_adjustment_view(item));
    }
    views
}

pub fn sum_redaction_adjustment_view(view: &RedactionAdjustmentView) -> Result<f64, String> {
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
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_floor_value + parsed_weight)
}
