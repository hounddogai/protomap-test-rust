use super::*;

use acme_kit::text;

/// How screens and exports show a regional page draft.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPageDraftView {
    pub code: String,
    pub title_text: String,
    pub weight: String,
    pub name: String,
    pub principal: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_regional_page_draft_view(item: &RegionalPageDraft) -> RegionalPageDraftView {
    RegionalPageDraftView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        name: text::truncate(&item.name, 80),
        principal: item.principal.clone(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_page_draft_views(items: &[RegionalPageDraft]) -> Vec<RegionalPageDraftView> {
    let mut views: Vec<RegionalPageDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_page_draft_view(item));
    }
    views
}

pub fn sum_regional_page_draft_view(view: &RegionalPageDraftView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_weight)
}
