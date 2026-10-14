use super::*;

use acme_kit::text;

/// How screens and exports show a silence window.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceWindowView {
    pub code_text: String,
    pub label: String,
    pub margin: String,
    pub description: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_silence_window_view(item: &SilenceWindow) -> SilenceWindowView {
    SilenceWindowView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        description: text::truncate(&item.description, 120),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_window_views(items: &[SilenceWindow]) -> Vec<SilenceWindowView> {
    let mut views: Vec<SilenceWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_window_view(item));
    }
    views
}

pub fn sum_silence_window_view(view: &SilenceWindowView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}
