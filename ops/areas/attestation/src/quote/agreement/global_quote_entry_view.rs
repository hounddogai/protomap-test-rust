use super::*;

use acme_kit::text;

/// How screens and exports show a global quote entry.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteEntryView {
    pub code: String,
    pub label_text: String,
    pub burst: String,
    pub starts_at: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_global_quote_entry_view(item: &GlobalQuoteEntry) -> GlobalQuoteEntryView {
    GlobalQuoteEntryView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        burst: item.burst.to_string(),
        starts_at: item.starts_at.to_string(),
        principal: text::truncate(&item.principal, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_quote_entry_views(items: &[GlobalQuoteEntry]) -> Vec<GlobalQuoteEntryView> {
    let mut views: Vec<GlobalQuoteEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_quote_entry_view(item));
    }
    views
}

pub fn sum_global_quote_entry_view(view: &GlobalQuoteEntryView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_starts_at as f64))
}
