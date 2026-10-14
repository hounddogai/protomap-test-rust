use super::*;

use acme_kit::text;

/// How screens and exports show a lenient canary slot.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanarySlotView {
    pub code_text: String,
    pub label: String,
    pub minimum: String,
    pub timezone_text: String,
    pub window_days: String,
    pub maximum: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_lenient_canary_slot_view(item: &LenientCanarySlot) -> LenientCanarySlotView {
    LenientCanarySlotView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        minimum: format!("{:.2}", item.minimum),
        timezone_text: text::truncate(&item.timezone, 40),
        window_days: item.window_days.to_string(),
        maximum: format!("{:.2}", item.maximum),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_canary_slot_views(items: &[LenientCanarySlot]) -> Vec<LenientCanarySlotView> {
    let mut views: Vec<LenientCanarySlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_canary_slot_view(item));
    }
    views
}

pub fn sum_lenient_canary_slot_view(view: &LenientCanarySlotView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_minimum + (parsed_window_days as f64) + parsed_maximum)
}
