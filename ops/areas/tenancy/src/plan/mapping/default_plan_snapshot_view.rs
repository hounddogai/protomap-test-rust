use super::*;

use acme_kit::text;

/// How screens and exports show a default plan snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPlanSnapshotView {
    pub code: String,
    pub name: String,
    pub version_text: String,
    pub cipher: String,
    pub ceiling_text: String,
    pub owner_id: String,
    pub updated_at_text: String,
    pub channel_text: String,
    pub status_label: String,
}

pub fn to_default_plan_snapshot_view(item: &DefaultPlanSnapshot) -> DefaultPlanSnapshotView {
    DefaultPlanSnapshotView {
        code: item.code.clone(),
        name: item.name.clone(),
        version_text: item.version.to_string(),
        cipher: text::truncate(&item.cipher, 120),
        ceiling_text: format!("{:.2}", item.ceiling),
        owner_id: item.owner_id.clone(),
        updated_at_text: item.updated_at.to_string(),
        channel_text: text::truncate(&item.channel, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_plan_snapshot_views(items: &[DefaultPlanSnapshot]) -> Vec<DefaultPlanSnapshotView> {
    let mut views: Vec<DefaultPlanSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_plan_snapshot_view(item));
    }
    views
}

pub fn sum_default_plan_snapshot_view(view: &DefaultPlanSnapshotView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_version as f64 + parsed_ceiling + (parsed_updated_at as f64))
}
