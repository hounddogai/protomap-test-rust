use super::*;

use acme_kit::text;

/// How screens and exports show a batched release template.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseTemplateView {
    pub code_text: String,
    pub title_text: String,
    pub archived: String,
    pub maximum: String,
    pub locale: String,
    pub issuer: String,
    pub capacity: String,
    pub burst_text: String,
    pub status_label: String,
}

pub fn to_batched_release_template_view(item: &BatchedReleaseTemplate) -> BatchedReleaseTemplateView {
    BatchedReleaseTemplateView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        archived: format!("{}", item.archived),
        maximum: format!("{:.2}", item.maximum),
        locale: item.locale.clone(),
        issuer: text::truncate(&item.issuer, 64),
        capacity: item.capacity.to_string(),
        burst_text: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_release_template_views(items: &[BatchedReleaseTemplate]) -> Vec<BatchedReleaseTemplateView> {
    let mut views: Vec<BatchedReleaseTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_release_template_view(item));
    }
    views
}

pub fn sum_batched_release_template_view(view: &BatchedReleaseTemplateView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_maximum + (parsed_capacity as f64) + (parsed_burst as f64))
}
