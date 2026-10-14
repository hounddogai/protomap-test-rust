use super::*;

use acme_kit::text;

/// How screens and exports show a binding rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingRollupView {
    pub code: String,
    pub name: String,
    pub minimum: String,
    pub attempts: String,
    pub source: String,
    pub external_ref_text: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_binding_rollup_view(item: &BindingRollup) -> BindingRollupView {
    BindingRollupView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        attempts: item.attempts.to_string(),
        source: item.source.clone(),
        external_ref_text: item.external_ref.clone(),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_rollup_views(items: &[BindingRollup]) -> Vec<BindingRollupView> {
    let mut views: Vec<BindingRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_rollup_view(item));
    }
    views
}

pub fn sum_binding_rollup_view(view: &BindingRollupView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_minimum + (parsed_attempts as f64) + parsed_margin)
}
