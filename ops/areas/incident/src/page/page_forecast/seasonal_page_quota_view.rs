use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal page quota.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPageQuotaView {
    pub code: String,
    pub label: String,
    pub cluster_text: String,
    pub priority: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_seasonal_page_quota_view(item: &SeasonalPageQuota) -> SeasonalPageQuotaView {
    SeasonalPageQuotaView {
        code: item.code.clone(),
        label: item.label.clone(),
        cluster_text: item.cluster.clone(),
        priority: item.priority.to_string(),
        notes: text::truncate(&item.notes, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_page_quota_views(items: &[SeasonalPageQuota]) -> Vec<SeasonalPageQuotaView> {
    let mut views: Vec<SeasonalPageQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_page_quota_view(item));
    }
    views
}

pub fn sum_seasonal_page_quota_view(view: &SeasonalPageQuotaView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}
