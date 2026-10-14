use super::*;

use acme_kit::text;

/// How screens and exports show a overage summary.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageSummaryView {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub margin: String,
    pub ratio: String,
    pub starts_at_text: String,
    pub visible_text: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_overage_summary_view(item: &OverageSummary) -> OverageSummaryView {
    OverageSummaryView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        timezone: text::truncate(&item.timezone, 64),
        margin: format!("{:.2}", item.margin),
        ratio: format!("{:.2}", item.ratio),
        starts_at_text: item.starts_at.to_string(),
        visible_text: format!("{}", item.visible),
        checksum_text: text::truncate(&item.checksum, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_summary_views(items: &[OverageSummary]) -> Vec<OverageSummaryView> {
    let mut views: Vec<OverageSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_summary_view(item));
    }
    views
}

pub fn sum_overage_summary_view(view: &OverageSummaryView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_margin + parsed_ratio + (parsed_starts_at as f64))
}
