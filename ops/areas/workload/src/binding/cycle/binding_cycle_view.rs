use super::*;

use acme_kit::text;

/// How screens and exports show a binding cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingCycleView {
    pub code: String,
    pub label_text: String,
    pub phone: String,
    pub notes: String,
    pub category: String,
    pub margin: String,
    pub quota_units_text: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_binding_cycle_view(item: &BindingCycle) -> BindingCycleView {
    BindingCycleView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        phone: item.phone.clone(),
        notes: item.notes.clone(),
        category: text::truncate(&item.category, 40),
        margin: format!("{:.2}", item.margin),
        quota_units_text: item.quota_units.to_string(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_cycle_views(items: &[BindingCycle]) -> Vec<BindingCycleView> {
    let mut views: Vec<BindingCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_cycle_view(item));
    }
    views
}

pub fn sum_binding_cycle_view(view: &BindingCycleView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_margin + (parsed_quota_units as f64) + (parsed_batch_size as f64))
}
