use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive page variance.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePageVarianceView {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub discount_text: String,
    pub summary_text: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_adaptive_page_variance_view(item: &AdaptivePageVariance) -> AdaptivePageVarianceView {
    AdaptivePageVarianceView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 120),
        unit: text::truncate(&item.unit, 40),
        discount_text: format!("{:.2}", item.discount),
        summary_text: item.summary.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_page_variance_views(items: &[AdaptivePageVariance]) -> Vec<AdaptivePageVarianceView> {
    let mut views: Vec<AdaptivePageVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_page_variance_view(item));
    }
    views
}

pub fn sum_adaptive_page_variance_view(view: &AdaptivePageVarianceView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_discount + (parsed_revision as f64))
}
