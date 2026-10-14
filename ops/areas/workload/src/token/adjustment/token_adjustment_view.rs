use super::*;

use acme_kit::text;

/// How screens and exports show a token adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenAdjustmentView {
    pub code: String,
    pub name: String,
    pub margin: String,
    pub maximum: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_token_adjustment_view(item: &TokenAdjustment) -> TokenAdjustmentView {
    TokenAdjustmentView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        margin: format!("{:.2}", item.margin),
        maximum: format!("{:.2}", item.maximum),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_adjustment_views(items: &[TokenAdjustment]) -> Vec<TokenAdjustmentView> {
    let mut views: Vec<TokenAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_adjustment_view(item));
    }
    views
}

pub fn sum_token_adjustment_view(view: &TokenAdjustmentView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_margin + parsed_maximum)
}
