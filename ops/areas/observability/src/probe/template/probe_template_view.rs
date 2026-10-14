use super::*;

use acme_kit::text;

/// How screens and exports show a probe template.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTemplateView {
    pub code: String,
    pub label: String,
    pub quota_units: String,
    pub floor_value_text: String,
    pub priority_text: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_probe_template_view(item: &ProbeTemplate) -> ProbeTemplateView {
    ProbeTemplateView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        priority_text: item.priority.to_string(),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_template_views(items: &[ProbeTemplate]) -> Vec<ProbeTemplateView> {
    let mut views: Vec<ProbeTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_template_view(item));
    }
    views
}

pub fn sum_probe_template_view(view: &ProbeTemplateView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_floor_value + (parsed_priority as f64))
}
