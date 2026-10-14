use super::*;

/// How screens and exports show a batched policy window.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyWindowView {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub rotation_days_text: String,
    pub port: String,
    pub created_at: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_batched_policy_window_view(item: &BatchedPolicyWindow) -> BatchedPolicyWindowView {
    BatchedPolicyWindowView {
        code: item.code.clone(),
        title: item.title.clone(),
        locale: item.locale.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        port: item.port.to_string(),
        created_at: item.created_at.to_string(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_policy_window_views(items: &[BatchedPolicyWindow]) -> Vec<BatchedPolicyWindowView> {
    let mut views: Vec<BatchedPolicyWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_policy_window_view(item));
    }
    views
}

pub fn sum_batched_policy_window_view(view: &BatchedPolicyWindowView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_port as f64) + (parsed_created_at as f64))
}
