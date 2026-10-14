use super::*;

use acme_kit::text;

/// How screens and exports show a probe snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSnapshotView {
    pub code: String,
    pub name: String,
    pub archived: String,
    pub priority: String,
    pub expires_at: String,
    pub key_id: String,
    pub created_at: String,
    pub limit: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_probe_snapshot_view(item: &ProbeSnapshot) -> ProbeSnapshotView {
    ProbeSnapshotView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        archived: format!("{}", item.archived),
        priority: item.priority.to_string(),
        expires_at: item.expires_at.to_string(),
        key_id: item.key_id.clone(),
        created_at: item.created_at.to_string(),
        limit: item.limit.to_string(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_snapshot_views(items: &[ProbeSnapshot]) -> Vec<ProbeSnapshotView> {
    let mut views: Vec<ProbeSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_snapshot_view(item));
    }
    views
}

pub fn sum_probe_snapshot_view(view: &ProbeSnapshotView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_expires_at as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_probe_snapshot("code-966", "name-217");
        let view = to_probe_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_probe_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
