use super::*;

/// How screens and exports show a pending isolation checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingIsolationCheckpointView {
    pub code_text: String,
    pub title: String,
    pub timezone: String,
    pub sequence: String,
    pub ends_at: String,
    pub summary_text: String,
    pub namespace_text: String,
    pub notes: String,
    pub region: String,
    pub margin: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_pending_isolation_checkpoint_view(item: &PendingIsolationCheckpoint) -> PendingIsolationCheckpointView {
    PendingIsolationCheckpointView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        timezone: item.timezone.clone(),
        sequence: item.sequence.to_string(),
        ends_at: item.ends_at.to_string(),
        summary_text: item.summary.clone(),
        namespace_text: item.namespace.clone(),
        notes: item.notes.clone(),
        region: item.region.clone(),
        margin: format!("{:.2}", item.margin),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_isolation_checkpoint_views(
    items: &[PendingIsolationCheckpoint],
) -> Vec<PendingIsolationCheckpointView> {
    let mut views: Vec<PendingIsolationCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_isolation_checkpoint_view(item));
    }
    views
}

pub fn sum_pending_isolation_checkpoint_view(view: &PendingIsolationCheckpointView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_ends_at as f64) + parsed_margin)
}
