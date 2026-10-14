use super::*;

use acme_kit::text;

/// How screens and exports show a default binding threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBindingThresholdView {
    pub code_text: String,
    pub label: String,
    pub active_text: String,
    pub phone_text: String,
    pub window_days: String,
    pub cost_text: String,
    pub ratio: String,
    pub fingerprint: String,
    pub channel: String,
    pub port: String,
    pub version_text: String,
    pub status_label: String,
}

pub fn to_default_binding_threshold_view(item: &DefaultBindingThreshold) -> DefaultBindingThresholdView {
    DefaultBindingThresholdView {
        code_text: item.code.clone(),
        label: text::truncate(&item.label, 64),
        active_text: format!("{}", item.active),
        phone_text: item.phone.clone(),
        window_days: item.window_days.to_string(),
        cost_text: format!("{:.2}", item.cost),
        ratio: format!("{:.2}", item.ratio),
        fingerprint: item.fingerprint.clone(),
        channel: item.channel.clone(),
        port: item.port.to_string(),
        version_text: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_binding_threshold_views(items: &[DefaultBindingThreshold]) -> Vec<DefaultBindingThresholdView> {
    let mut views: Vec<DefaultBindingThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_binding_threshold_view(item));
    }
    views
}

pub fn sum_default_binding_threshold_view(view: &DefaultBindingThresholdView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_cost + parsed_ratio)
}
