use super::*;

use acme_kit::text;

/// How screens and exports show a tiered overage entry.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageEntryView {
    pub code: String,
    pub title_text: String,
    pub segment: String,
    pub checksum: String,
    pub scope_text: String,
    pub name: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_tiered_overage_entry_view(item: &TieredOverageEntry) -> TieredOverageEntryView {
    TieredOverageEntryView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        segment: text::truncate(&item.segment, 120),
        checksum: item.checksum.clone(),
        scope_text: item.scope.clone(),
        name: item.name.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_overage_entry_views(items: &[TieredOverageEntry]) -> Vec<TieredOverageEntryView> {
    let mut views: Vec<TieredOverageEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_overage_entry_view(item));
    }
    views
}

pub fn sum_tiered_overage_entry_view(view: &TieredOverageEntryView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}
