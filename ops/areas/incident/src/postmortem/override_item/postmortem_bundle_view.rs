use super::*;

/// How screens and exports show a postmortem bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBundleView {
    pub code_text: String,
    pub label: String,
    pub rate: String,
    pub window_days: String,
    pub visible_text: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_postmortem_bundle_view(item: &PostmortemBundle) -> PostmortemBundleView {
    PostmortemBundleView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        window_days: item.window_days.to_string(),
        visible_text: format!("{}", item.visible),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_bundle_views(items: &[PostmortemBundle]) -> Vec<PostmortemBundleView> {
    let mut views: Vec<PostmortemBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_bundle_view(item));
    }
    views
}

pub fn sum_postmortem_bundle_view(view: &PostmortemBundleView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_rate + (parsed_window_days as f64))
}
