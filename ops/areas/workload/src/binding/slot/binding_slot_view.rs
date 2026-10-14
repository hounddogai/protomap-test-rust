use super::*;

use acme_kit::text;

/// How screens and exports show a binding slot.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingSlotView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub quantity_text: String,
    pub minimum_text: String,
    pub segment: String,
    pub audience: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_binding_slot_view(item: &BindingSlot) -> BindingSlotView {
    BindingSlotView {
        code: item.code.clone(),
        title: item.title.clone(),
        country: item.country.clone(),
        quantity_text: item.quantity.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        segment: item.segment.clone(),
        audience: text::truncate(&item.audience, 120),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_slot_views(items: &[BindingSlot]) -> Vec<BindingSlotView> {
    let mut views: Vec<BindingSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_slot_view(item));
    }
    views
}

pub fn sum_binding_slot_view(view: &BindingSlotView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_minimum + parsed_margin)
}
