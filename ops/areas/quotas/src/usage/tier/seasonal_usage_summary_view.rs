use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal usage summary.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalUsageSummaryView {
    pub code_text: String,
    pub title: String,
    pub approved: String,
    pub segment: String,
    pub category: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_seasonal_usage_summary_view(item: &SeasonalUsageSummary) -> SeasonalUsageSummaryView {
    SeasonalUsageSummaryView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        approved: format!("{}", item.approved),
        segment: text::truncate(&item.segment, 80),
        category: item.category.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_usage_summary_views(items: &[SeasonalUsageSummary]) -> Vec<SeasonalUsageSummaryView> {
    let mut views: Vec<SeasonalUsageSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_usage_summary_view(item));
    }
    views
}

pub fn sum_seasonal_usage_summary_view(view: &SeasonalUsageSummaryView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}
