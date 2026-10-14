use super::*;

/// How screens and exports show a manual nonce checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualNonceCheckpointView {
    pub code_text: String,
    pub name: String,
    pub summary: String,
    pub updated_at_text: String,
    pub cluster_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_manual_nonce_checkpoint_view(item: &ManualNonceCheckpoint) -> ManualNonceCheckpointView {
    ManualNonceCheckpointView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        summary: item.summary.clone(),
        updated_at_text: item.updated_at.to_string(),
        cluster_text: item.cluster.clone(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_nonce_checkpoint_views(items: &[ManualNonceCheckpoint]) -> Vec<ManualNonceCheckpointView> {
    let mut views: Vec<ManualNonceCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_nonce_checkpoint_view(item));
    }
    views
}

pub fn sum_manual_nonce_checkpoint_view(view: &ManualNonceCheckpointView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_ratio)
}
