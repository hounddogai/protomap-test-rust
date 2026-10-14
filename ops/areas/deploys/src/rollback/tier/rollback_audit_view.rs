use super::*;

use acme_kit::text;

/// How screens and exports show a rollback audit.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAuditView {
    pub code: String,
    pub label_text: String,
    pub country_text: String,
    pub cluster_text: String,
    pub version_text: String,
    pub window_days: String,
    pub starts_at: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_rollback_audit_view(item: &RollbackAudit) -> RollbackAuditView {
    RollbackAuditView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 120),
        country_text: item.country.clone(),
        cluster_text: item.cluster.clone(),
        version_text: item.version.to_string(),
        window_days: item.window_days.to_string(),
        starts_at: item.starts_at.to_string(),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_audit_views(items: &[RollbackAudit]) -> Vec<RollbackAuditView> {
    let mut views: Vec<RollbackAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_audit_view(item));
    }
    views
}

pub fn sum_rollback_audit_view(view: &RollbackAuditView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_version as f64 + (parsed_window_days as f64) + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_audit("code-269", "label-779");
        let view = to_rollback_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
