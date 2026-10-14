use super::*;

use acme_kit::text;

/// How screens and exports show a tenant snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantSnapshotView {
    pub code: String,
    pub name: String,
    pub rotation_days: String,
    pub source: String,
    pub priority: String,
    pub locale: String,
    pub limit: String,
    pub starts_at_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_tenant_snapshot_view(item: &TenantSnapshot) -> TenantSnapshotView {
    TenantSnapshotView {
        code: item.code.clone(),
        name: item.name.clone(),
        rotation_days: item.rotation_days.to_string(),
        source: item.source.clone(),
        priority: item.priority.to_string(),
        locale: item.locale.clone(),
        limit: item.limit.to_string(),
        starts_at_text: item.starts_at.to_string(),
        audience: text::truncate(&item.audience, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_snapshot_views(items: &[TenantSnapshot]) -> Vec<TenantSnapshotView> {
    let mut views: Vec<TenantSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_snapshot_view(item));
    }
    views
}

pub fn sum_tenant_snapshot_view(view: &TenantSnapshotView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_priority as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_snapshot("code-850", "name-442");
        let view = to_tenant_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
