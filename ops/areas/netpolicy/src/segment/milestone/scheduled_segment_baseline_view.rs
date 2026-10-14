use super::*;

/// How screens and exports show a scheduled segment baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentBaselineView {
    pub code_text: String,
    pub name_text: String,
    pub port_text: String,
    pub capacity: String,
    pub amount_text: String,
    pub phone: String,
    pub owner_id: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_scheduled_segment_baseline_view(item: &ScheduledSegmentBaseline) -> ScheduledSegmentBaselineView {
    ScheduledSegmentBaselineView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        port_text: item.port.to_string(),
        capacity: item.capacity.to_string(),
        amount_text: format!("{:.2}", item.amount),
        phone: item.phone.clone(),
        owner_id: item.owner_id.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_segment_baseline_views(items: &[ScheduledSegmentBaseline]) -> Vec<ScheduledSegmentBaselineView> {
    let mut views: Vec<ScheduledSegmentBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_segment_baseline_view(item));
    }
    views
}

pub fn sum_scheduled_segment_baseline_view(view: &ScheduledSegmentBaselineView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_port as f64 + (parsed_capacity as f64) + parsed_amount)
}
