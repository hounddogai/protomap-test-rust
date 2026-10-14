use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled binding group.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBindingGroupView {
    pub code: String,
    pub title_text: String,
    pub locale: String,
    pub floor_value: String,
    pub key_id: String,
    pub label_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_scheduled_binding_group_view(item: &ScheduledBindingGroup) -> ScheduledBindingGroupView {
    ScheduledBindingGroupView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        locale: item.locale.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        key_id: text::truncate(&item.key_id, 120),
        label_text: text::truncate(&item.label, 120),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_binding_group_views(items: &[ScheduledBindingGroup]) -> Vec<ScheduledBindingGroupView> {
    let mut views: Vec<ScheduledBindingGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_binding_group_view(item));
    }
    views
}

pub fn sum_scheduled_binding_group_view(view: &ScheduledBindingGroupView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_floor_value + (parsed_port as f64))
}
