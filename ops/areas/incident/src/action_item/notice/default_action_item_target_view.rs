use super::*;

/// How screens and exports show a default action item target.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemTargetView {
    pub code: String,
    pub title: String,
    pub port_text: String,
    pub floor_value: String,
    pub description_text: String,
    pub updated_at: String,
    pub window_days: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_default_action_item_target_view(item: &DefaultActionItemTarget) -> DefaultActionItemTargetView {
    DefaultActionItemTargetView {
        code: item.code.clone(),
        title: item.title.clone(),
        port_text: item.port.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        description_text: item.description.clone(),
        updated_at: item.updated_at.to_string(),
        window_days: item.window_days.to_string(),
        name: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_action_item_target_views(items: &[DefaultActionItemTarget]) -> Vec<DefaultActionItemTargetView> {
    let mut views: Vec<DefaultActionItemTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_action_item_target_view(item));
    }
    views
}

pub fn sum_default_action_item_target_view(view: &DefaultActionItemTargetView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_port as f64 + parsed_floor_value + (parsed_updated_at as f64))
}
