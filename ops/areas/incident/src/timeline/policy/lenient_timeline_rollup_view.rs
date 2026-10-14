use super::*;

/// How screens and exports show a lenient timeline rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineRollupView {
    pub code: String,
    pub label_text: String,
    pub issuer: String,
    pub expires_at: String,
    pub sequence: String,
    pub active: String,
    pub floor_value: String,
    pub principal_text: String,
    pub status_label: String,
}

pub fn to_lenient_timeline_rollup_view(item: &LenientTimelineRollup) -> LenientTimelineRollupView {
    LenientTimelineRollupView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        issuer: item.issuer.clone(),
        expires_at: item.expires_at.to_string(),
        sequence: item.sequence.to_string(),
        active: format!("{}", item.active),
        floor_value: format!("{:.2}", item.floor_value),
        principal_text: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_timeline_rollup_views(items: &[LenientTimelineRollup]) -> Vec<LenientTimelineRollupView> {
    let mut views: Vec<LenientTimelineRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_timeline_rollup_view(item));
    }
    views
}

pub fn sum_lenient_timeline_rollup_view(view: &LenientTimelineRollupView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_sequence as f64) + parsed_floor_value)
}
