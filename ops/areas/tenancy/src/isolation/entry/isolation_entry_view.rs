use super::*;

use acme_kit::text;

/// How screens and exports show a isolation entry.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEntryView {
    pub code: String,
    pub label: String,
    pub source: String,
    pub principal: String,
    pub ttl_seconds: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_isolation_entry_view(item: &IsolationEntry) -> IsolationEntryView {
    IsolationEntryView {
        code: item.code.clone(),
        label: item.label.clone(),
        source: text::truncate(&item.source, 64),
        principal: item.principal.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_entry_views(items: &[IsolationEntry]) -> Vec<IsolationEntryView> {
    let mut views: Vec<IsolationEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_entry_view(item));
    }
    views
}

pub fn sum_isolation_entry_view(view: &IsolationEntryView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_retries as f64))
}
