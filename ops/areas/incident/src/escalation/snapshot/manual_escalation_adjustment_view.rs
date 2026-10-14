use super::*;

/// How screens and exports show a manual escalation adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualEscalationAdjustmentView {
    pub code_text: String,
    pub title: String,
    pub active: String,
    pub pinned_text: String,
    pub burst_text: String,
    pub archived: String,
    pub floor_value: String,
    pub rate: String,
    pub channel: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_manual_escalation_adjustment_view(item: &ManualEscalationAdjustment) -> ManualEscalationAdjustmentView {
    ManualEscalationAdjustmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        active: format!("{}", item.active),
        pinned_text: format!("{}", item.pinned),
        burst_text: item.burst.to_string(),
        archived: format!("{}", item.archived),
        floor_value: format!("{:.2}", item.floor_value),
        rate: format!("{:.2}", item.rate),
        channel: item.channel.clone(),
        name: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_escalation_adjustment_views(
    items: &[ManualEscalationAdjustment],
) -> Vec<ManualEscalationAdjustmentView> {
    let mut views: Vec<ManualEscalationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_escalation_adjustment_view(item));
    }
    views
}

pub fn sum_manual_escalation_adjustment_view(view: &ManualEscalationAdjustmentView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_burst as f64 + parsed_floor_value + parsed_rate)
}
