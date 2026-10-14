use super::*;

/// How screens and exports show a segment adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAdjustmentView {
    pub code_text: String,
    pub name_text: String,
    pub cipher_text: String,
    pub active: String,
    pub port: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_segment_adjustment_view(item: &SegmentAdjustment) -> SegmentAdjustmentView {
    SegmentAdjustmentView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        cipher_text: item.cipher.clone(),
        active: format!("{}", item.active),
        port: item.port.to_string(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_segment_adjustment_views(items: &[SegmentAdjustment]) -> Vec<SegmentAdjustmentView> {
    let mut views: Vec<SegmentAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_segment_adjustment_view(item));
    }
    views
}

pub fn sum_segment_adjustment_view(view: &SegmentAdjustmentView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_port as f64 + parsed_budget)
}
