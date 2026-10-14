use super::*;

/// How screens and exports show a global silence window.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceWindowView {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub notes: String,
    pub country: String,
    pub active: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_global_silence_window_view(item: &GlobalSilenceWindow) -> GlobalSilenceWindowView {
    GlobalSilenceWindowView {
        code: item.code.clone(),
        label: item.label.clone(),
        audience: item.audience.clone(),
        notes: item.notes.clone(),
        country: item.country.clone(),
        active: format!("{}", item.active),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_silence_window_views(items: &[GlobalSilenceWindow]) -> Vec<GlobalSilenceWindowView> {
    let mut views: Vec<GlobalSilenceWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_silence_window_view(item));
    }
    views
}

pub fn sum_global_silence_window_view(view: &GlobalSilenceWindowView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
