use super::*;

use acme_kit::text;

/// How screens and exports show a page exception.
#[derive(Debug, Clone, PartialEq)]
pub struct PageExceptionView {
    pub code: String,
    pub name_text: String,
    pub external_ref_text: String,
    pub country: String,
    pub minimum: String,
    pub summary_text: String,
    pub status_label: String,
}

pub fn to_page_exception_view(item: &PageException) -> PageExceptionView {
    PageExceptionView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        external_ref_text: text::truncate(&item.external_ref, 64),
        country: item.country.clone(),
        minimum: format!("{:.2}", item.minimum),
        summary_text: item.summary.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_exception_views(items: &[PageException]) -> Vec<PageExceptionView> {
    let mut views: Vec<PageExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_exception_view(item));
    }
    views
}

pub fn sum_page_exception_view(view: &PageExceptionView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}
