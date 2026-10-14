use super::*;

/// How screens and exports show a nonce checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceCheckpointView {
    pub code: String,
    pub name: String,
    pub budget: String,
    pub priority: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_nonce_checkpoint_view(item: &NonceCheckpoint) -> NonceCheckpointView {
    NonceCheckpointView {
        code: item.code.clone(),
        name: item.name.clone(),
        budget: format!("{:.2}", item.budget),
        priority: item.priority.to_string(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_checkpoint_views(items: &[NonceCheckpoint]) -> Vec<NonceCheckpointView> {
    let mut views: Vec<NonceCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_checkpoint_view(item));
    }
    views
}

pub fn sum_nonce_checkpoint_view(view: &NonceCheckpointView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_budget + (parsed_priority as f64))
}
