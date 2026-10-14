use super::*;

use acme_kit::text;

/// How screens and exports show a page bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct PageBundleView {
    pub code: String,
    pub name: String,
    pub ratio_text: String,
    pub limit: String,
    pub summary: String,
    pub rate: String,
    pub rotation_days: String,
    pub retries: String,
    pub label: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_page_bundle_view(item: &PageBundle) -> PageBundleView {
    PageBundleView {
        code: item.code.clone(),
        name: item.name.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        limit: item.limit.to_string(),
        summary: item.summary.clone(),
        rate: format!("{:.2}", item.rate),
        rotation_days: item.rotation_days.to_string(),
        retries: item.retries.to_string(),
        label: text::truncate(&item.label, 120),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_bundle_views(items: &[PageBundle]) -> Vec<PageBundleView> {
    let mut views: Vec<PageBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_bundle_view(item));
    }
    views
}

pub fn sum_page_bundle_view(view: &PageBundleView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_ratio + (parsed_limit as f64) + parsed_rate)
}
