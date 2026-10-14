use super::*;

use acme_kit::text;

/// How screens and exports show a rolling rollback snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRollbackSnapshotView {
    pub code_text: String,
    pub name: String,
    pub updated_at: String,
    pub capacity_text: String,
    pub weight: String,
    pub minimum: String,
    pub region: String,
    pub version_text: String,
    pub burst: String,
    pub starts_at: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_rolling_rollback_snapshot_view(item: &RollingRollbackSnapshot) -> RollingRollbackSnapshotView {
    RollingRollbackSnapshotView {
        code_text: text::truncate(&item.code, 64),
        name: item.name.clone(),
        updated_at: item.updated_at.to_string(),
        capacity_text: item.capacity.to_string(),
        weight: format!("{:.2}", item.weight),
        minimum: format!("{:.2}", item.minimum),
        region: text::truncate(&item.region, 80),
        version_text: item.version.to_string(),
        burst: item.burst.to_string(),
        starts_at: item.starts_at.to_string(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_rollback_snapshot_views(items: &[RollingRollbackSnapshot]) -> Vec<RollingRollbackSnapshotView> {
    let mut views: Vec<RollingRollbackSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_rollback_snapshot_view(item));
    }
    views
}

pub fn sum_rolling_rollback_snapshot_view(view: &RollingRollbackSnapshotView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_capacity as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_rollback_snapshot("code-435", "name-880");
        let view = to_rolling_rollback_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_rollback_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
