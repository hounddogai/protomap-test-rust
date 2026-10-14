use super::*;

/// How screens and exports show a batched allowlist snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistSnapshotView {
    pub code: String,
    pub label: String,
    pub region: String,
    pub pinned_text: String,
    pub updated_at_text: String,
    pub limit_text: String,
    pub capacity_text: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_batched_allowlist_snapshot_view(item: &BatchedAllowlistSnapshot) -> BatchedAllowlistSnapshotView {
    BatchedAllowlistSnapshotView {
        code: item.code.clone(),
        label: item.label.clone(),
        region: item.region.clone(),
        pinned_text: format!("{}", item.pinned),
        updated_at_text: item.updated_at.to_string(),
        limit_text: item.limit.to_string(),
        capacity_text: item.capacity.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_allowlist_snapshot_views(items: &[BatchedAllowlistSnapshot]) -> Vec<BatchedAllowlistSnapshotView> {
    let mut views: Vec<BatchedAllowlistSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_allowlist_snapshot_view(item));
    }
    views
}

pub fn sum_batched_allowlist_snapshot_view(view: &BatchedAllowlistSnapshotView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_limit as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_allowlist_snapshot("code-198", "label-880");
        let view = to_batched_allowlist_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_allowlist_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
