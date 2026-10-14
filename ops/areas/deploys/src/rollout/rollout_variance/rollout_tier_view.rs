use super::*;

use acme_kit::text;

/// How screens and exports show a rollout tier.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTierView {
    pub code: String,
    pub title: String,
    pub batch_size: String,
    pub quota_units: String,
    pub segment: String,
    pub ratio_text: String,
    pub contact_email: String,
    pub updated_at: String,
    pub checksum: String,
    pub cluster: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_rollout_tier_view(item: &RolloutTier) -> RolloutTierView {
    RolloutTierView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        batch_size: item.batch_size.to_string(),
        quota_units: item.quota_units.to_string(),
        segment: item.segment.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        contact_email: item.contact_email.clone(),
        updated_at: item.updated_at.to_string(),
        checksum: item.checksum.clone(),
        cluster: item.cluster.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_tier_views(items: &[RolloutTier]) -> Vec<RolloutTierView> {
    let mut views: Vec<RolloutTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_tier_view(item));
    }
    views
}

pub fn sum_rollout_tier_view(view: &RolloutTierView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_quota_units as f64) + parsed_ratio)
}
