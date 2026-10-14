use super::*;

use acme_kit::text;

/// How screens and exports show a tiered postmortem cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPostmortemCycleView {
    pub code_text: String,
    pub label: String,
    pub segment_text: String,
    pub duration_minutes: String,
    pub host_text: String,
    pub floor_value_text: String,
    pub description: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_tiered_postmortem_cycle_view(item: &TieredPostmortemCycle) -> TieredPostmortemCycleView {
    TieredPostmortemCycleView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 80),
        segment_text: item.segment.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        host_text: item.host.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        description: item.description.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_postmortem_cycle_views(items: &[TieredPostmortemCycle]) -> Vec<TieredPostmortemCycleView> {
    let mut views: Vec<TieredPostmortemCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_postmortem_cycle_view(item));
    }
    views
}

pub fn sum_tiered_postmortem_cycle_view(view: &TieredPostmortemCycleView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_duration_minutes as f64 + parsed_floor_value + (parsed_burst as f64))
}
