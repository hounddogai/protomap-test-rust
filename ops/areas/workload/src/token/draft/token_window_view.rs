use super::*;

use acme_kit::text;

/// How screens and exports show a token window.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWindowView {
    pub code_text: String,
    pub label_text: String,
    pub locale: String,
    pub ttl_seconds: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_token_window_view(item: &TokenWindow) -> TokenWindowView {
    TokenWindowView {
        code_text: text::truncate(&item.code, 80),
        label_text: text::truncate(&item.label, 64),
        locale: item.locale.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_window_views(items: &[TokenWindow]) -> Vec<TokenWindowView> {
    let mut views: Vec<TokenWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_window_view(item));
    }
    views
}

pub fn sum_token_window_view(view: &TokenWindowView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_minimum)
}
