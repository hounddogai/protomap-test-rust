use super::*;

/// How screens and exports show a global sink slot.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSinkSlotView {
    pub code: String,
    pub name: String,
    pub country: String,
    pub floor_value_text: String,
    pub score: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_global_sink_slot_view(item: &GlobalSinkSlot) -> GlobalSinkSlotView {
    GlobalSinkSlotView {
        code: item.code.clone(),
        name: item.name.clone(),
        country: item.country.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        score: format!("{:.2}", item.score),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_sink_slot_views(items: &[GlobalSinkSlot]) -> Vec<GlobalSinkSlotView> {
    let mut views: Vec<GlobalSinkSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_sink_slot_view(item));
    }
    views
}

pub fn sum_global_sink_slot_view(view: &GlobalSinkSlotView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_floor_value + parsed_score + parsed_maximum)
}
