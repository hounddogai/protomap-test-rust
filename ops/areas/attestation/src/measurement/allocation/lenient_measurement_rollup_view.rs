use super::*;

use acme_kit::text;

/// How screens and exports show a lenient measurement rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientMeasurementRollupView {
    pub code_text: String,
    pub label_text: String,
    pub currency: String,
    pub issuer: String,
    pub external_ref: String,
    pub cipher: String,
    pub margin: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_lenient_measurement_rollup_view(item: &LenientMeasurementRollup) -> LenientMeasurementRollupView {
    LenientMeasurementRollupView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        currency: item.currency.clone(),
        issuer: item.issuer.clone(),
        external_ref: text::truncate(&item.external_ref, 40),
        cipher: item.cipher.clone(),
        margin: format!("{:.2}", item.margin),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_measurement_rollup_views(items: &[LenientMeasurementRollup]) -> Vec<LenientMeasurementRollupView> {
    let mut views: Vec<LenientMeasurementRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_measurement_rollup_view(item));
    }
    views
}

pub fn sum_lenient_measurement_rollup_view(view: &LenientMeasurementRollupView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}
