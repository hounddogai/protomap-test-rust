use super::*;

use acme_kit::text;

/// How screens and exports show a global zone digest.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalZoneDigestView {
    pub code: String,
    pub label: String,
    pub channel_text: String,
    pub floor_value: String,
    pub grace_days: String,
    pub segment: String,
    pub summary: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_global_zone_digest_view(item: &GlobalZoneDigest) -> GlobalZoneDigestView {
    GlobalZoneDigestView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        channel_text: item.channel.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        grace_days: item.grace_days.to_string(),
        segment: item.segment.clone(),
        summary: item.summary.clone(),
        owner_id: item.owner_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_zone_digest_views(items: &[GlobalZoneDigest]) -> Vec<GlobalZoneDigestView> {
    let mut views: Vec<GlobalZoneDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_zone_digest_view(item));
    }
    views
}

pub fn sum_global_zone_digest_view(view: &GlobalZoneDigestView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_floor_value + (parsed_grace_days as f64))
}
