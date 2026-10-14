use super::*;

/// How screens and exports show a fallback zone slot.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackZoneSlotView {
    pub code: String,
    pub label: String,
    pub grace_days_text: String,
    pub source_text: String,
    pub margin: String,
    pub updated_at: String,
    pub locale: String,
    pub audience_text: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_fallback_zone_slot_view(item: &FallbackZoneSlot) -> FallbackZoneSlotView {
    FallbackZoneSlotView {
        code: item.code.clone(),
        label: item.label.clone(),
        grace_days_text: item.grace_days.to_string(),
        source_text: item.source.clone(),
        margin: format!("{:.2}", item.margin),
        updated_at: item.updated_at.to_string(),
        locale: item.locale.clone(),
        audience_text: item.audience.clone(),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_zone_slot_views(items: &[FallbackZoneSlot]) -> Vec<FallbackZoneSlotView> {
    let mut views: Vec<FallbackZoneSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_zone_slot_view(item));
    }
    views
}

pub fn sum_fallback_zone_slot_view(view: &FallbackZoneSlotView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_margin + (parsed_updated_at as f64))
}
