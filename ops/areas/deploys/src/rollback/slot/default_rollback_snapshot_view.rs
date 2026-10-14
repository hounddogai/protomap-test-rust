use super::*;

use acme_kit::text;

/// How screens and exports show a default rollback snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackSnapshotView {
    pub code: String,
    pub label: String,
    pub visible: String,
    pub region: String,
    pub budget: String,
    pub burst_text: String,
    pub ceiling: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_default_rollback_snapshot_view(item: &DefaultRollbackSnapshot) -> DefaultRollbackSnapshotView {
    DefaultRollbackSnapshotView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 64),
        visible: format!("{}", item.visible),
        region: item.region.clone(),
        budget: format!("{:.2}", item.budget),
        burst_text: item.burst.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_rollback_snapshot_views(items: &[DefaultRollbackSnapshot]) -> Vec<DefaultRollbackSnapshotView> {
    let mut views: Vec<DefaultRollbackSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_rollback_snapshot_view(item));
    }
    views
}

pub fn sum_default_rollback_snapshot_view(view: &DefaultRollbackSnapshotView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_budget + (parsed_burst as f64) + parsed_ceiling)
}
