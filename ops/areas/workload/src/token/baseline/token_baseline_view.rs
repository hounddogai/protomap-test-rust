use super::*;

use acme_kit::text;

/// How screens and exports show a token baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenBaselineView {
    pub code_text: String,
    pub label_text: String,
    pub starts_at: String,
    pub budget: String,
    pub ratio: String,
    pub country: String,
    pub pinned: String,
    pub locale: String,
    pub active_text: String,
    pub summary: String,
    pub status_label: String,
}

pub fn to_token_baseline_view(item: &TokenBaseline) -> TokenBaselineView {
    TokenBaselineView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        starts_at: item.starts_at.to_string(),
        budget: format!("{:.2}", item.budget),
        ratio: format!("{:.2}", item.ratio),
        country: text::truncate(&item.country, 120),
        pinned: format!("{}", item.pinned),
        locale: item.locale.clone(),
        active_text: format!("{}", item.active),
        summary: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_token_baseline_views(items: &[TokenBaseline]) -> Vec<TokenBaselineView> {
    let mut views: Vec<TokenBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_token_baseline_view(item));
    }
    views
}

pub fn sum_token_baseline_view(view: &TokenBaselineView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_budget + parsed_ratio)
}
