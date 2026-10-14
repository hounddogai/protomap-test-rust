use super::*;

/// How screens and exports show a page digest.
#[derive(Debug, Clone, PartialEq)]
pub struct PageDigestView {
    pub code: String,
    pub name: String,
    pub ends_at: String,
    pub retries: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_page_digest_view(item: &PageDigest) -> PageDigestView {
    PageDigestView {
        code: item.code.clone(),
        name: item.name.clone(),
        ends_at: item.ends_at.to_string(),
        retries: item.retries.to_string(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_digest_views(items: &[PageDigest]) -> Vec<PageDigestView> {
    let mut views: Vec<PageDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_digest_view(item));
    }
    views
}

pub fn sum_page_digest_view(view: &PageDigestView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_retries as f64) + (parsed_grace_days as f64))
}
