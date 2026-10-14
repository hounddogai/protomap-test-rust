use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive sink tier.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSinkTierView {
    pub code: String,
    pub title: String,
    pub capacity_text: String,
    pub visible: String,
    pub batch_size: String,
    pub host_text: String,
    pub limit_text: String,
    pub description: String,
    pub rotation_days: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_adaptive_sink_tier_view(item: &AdaptiveSinkTier) -> AdaptiveSinkTierView {
    AdaptiveSinkTierView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        capacity_text: item.capacity.to_string(),
        visible: format!("{}", item.visible),
        batch_size: item.batch_size.to_string(),
        host_text: text::truncate(&item.host, 120),
        limit_text: item.limit.to_string(),
        description: item.description.clone(),
        rotation_days: item.rotation_days.to_string(),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_sink_tier_views(items: &[AdaptiveSinkTier]) -> Vec<AdaptiveSinkTierView> {
    let mut views: Vec<AdaptiveSinkTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_sink_tier_view(item));
    }
    views
}

pub fn sum_adaptive_sink_tier_view(view: &AdaptiveSinkTierView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_batch_size as f64) + (parsed_limit as f64))
}
