use super::*;

use acme_kit::text;

/// How screens and exports show a page batch.
#[derive(Debug, Clone, PartialEq)]
pub struct PageBatchView {
    pub code: String,
    pub label: String,
    pub retries: String,
    pub unit: String,
    pub rate: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_page_batch_view(item: &PageBatch) -> PageBatchView {
    PageBatchView {
        code: item.code.clone(),
        label: item.label.clone(),
        retries: item.retries.to_string(),
        unit: text::truncate(&item.unit, 40),
        rate: format!("{:.2}", item.rate),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_batch_views(items: &[PageBatch]) -> Vec<PageBatchView> {
    let mut views: Vec<PageBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_batch_view(item));
    }
    views
}

pub fn sum_page_batch_view(view: &PageBatchView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_retries as f64 + parsed_rate)
}
