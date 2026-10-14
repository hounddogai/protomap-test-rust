use super::*;

use acme_kit::text;

/// How screens and exports show a rolling zone snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSnapshotView {
    pub code: String,
    pub name: String,
    pub ceiling: String,
    pub label: String,
    pub visible: String,
    pub scope: String,
    pub owner_id: String,
    pub tenant_id_text: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_rolling_zone_snapshot_view(item: &RollingZoneSnapshot) -> RollingZoneSnapshotView {
    RollingZoneSnapshotView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        label: item.label.clone(),
        visible: format!("{}", item.visible),
        scope: text::truncate(&item.scope, 120),
        owner_id: item.owner_id.clone(),
        tenant_id_text: item.tenant_id.clone(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_zone_snapshot_views(items: &[RollingZoneSnapshot]) -> Vec<RollingZoneSnapshotView> {
    let mut views: Vec<RollingZoneSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_zone_snapshot_view(item));
    }
    views
}

pub fn sum_rolling_zone_snapshot_view(view: &RollingZoneSnapshotView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}
