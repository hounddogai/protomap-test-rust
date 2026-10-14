use super::*;

/// How screens and exports show a default segment checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSegmentCheckpointView {
    pub code_text: String,
    pub label_text: String,
    pub host: String,
    pub rotation_days: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_default_segment_checkpoint_view(item: &DefaultSegmentCheckpoint) -> DefaultSegmentCheckpointView {
    DefaultSegmentCheckpointView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        host: item.host.clone(),
        rotation_days: item.rotation_days.to_string(),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_segment_checkpoint_views(items: &[DefaultSegmentCheckpoint]) -> Vec<DefaultSegmentCheckpointView> {
    let mut views: Vec<DefaultSegmentCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_segment_checkpoint_view(item));
    }
    views
}

pub fn sum_default_segment_checkpoint_view(view: &DefaultSegmentCheckpointView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_updated_at as f64))
}
