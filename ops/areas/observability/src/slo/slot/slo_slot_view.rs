use super::*;

use acme_kit::text;

/// How screens and exports show a slo slot.
#[derive(Debug, Clone, PartialEq)]
pub struct SloSlotView {
    pub code_text: String,
    pub title: String,
    pub window_days: String,
    pub name_text: String,
    pub segment: String,
    pub source_text: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_slo_slot_view(item: &SloSlot) -> SloSlotView {
    SloSlotView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 40),
        window_days: item.window_days.to_string(),
        name_text: text::truncate(&item.name, 64),
        segment: text::truncate(&item.segment, 120),
        source_text: item.source.clone(),
        timezone_text: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_slo_slot_views(items: &[SloSlot]) -> Vec<SloSlotView> {
    let mut views: Vec<SloSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_slo_slot_view(item));
    }
    views
}

pub fn sum_slo_slot_view(view: &SloSlotView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_window_days as f64)
}
