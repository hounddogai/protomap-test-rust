use super::*;

use acme_kit::text;

/// How screens and exports show a binding bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingBundleView {
    pub code: String,
    pub title: String,
    pub rate: String,
    pub approved_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_binding_bundle_view(item: &BindingBundle) -> BindingBundleView {
    BindingBundleView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        approved_text: format!("{}", item.approved),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_bundle_views(items: &[BindingBundle]) -> Vec<BindingBundleView> {
    let mut views: Vec<BindingBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_bundle_view(item));
    }
    views
}

pub fn sum_binding_bundle_view(view: &BindingBundleView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_rate + (parsed_version as f64))
}
