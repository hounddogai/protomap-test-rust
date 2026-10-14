use super::*;

use acme_kit::text;

/// How screens and exports show a fallback token notice.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenNoticeView {
    pub code: String,
    pub title: String,
    pub cost: String,
    pub scope_text: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_fallback_token_notice_view(item: &FallbackTokenNotice) -> FallbackTokenNoticeView {
    FallbackTokenNoticeView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        cost: format!("{:.2}", item.cost),
        scope_text: item.scope.clone(),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_token_notice_views(items: &[FallbackTokenNotice]) -> Vec<FallbackTokenNoticeView> {
    let mut views: Vec<FallbackTokenNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_token_notice_view(item));
    }
    views
}

pub fn sum_fallback_token_notice_view(view: &FallbackTokenNoticeView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}
