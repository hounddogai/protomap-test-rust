use super::*;

use acme_kit::text;

/// How screens and exports show a legacy grant audit.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantAuditView {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub cluster_text: String,
    pub label_text: String,
    pub title: String,
    pub unit: String,
    pub cost: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_legacy_grant_audit_view(item: &LegacyGrantAudit) -> LegacyGrantAuditView {
    LegacyGrantAuditView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        owner_id: item.owner_id.clone(),
        cluster_text: item.cluster.clone(),
        label_text: item.label.clone(),
        title: item.title.clone(),
        unit: text::truncate(&item.unit, 64),
        cost: format!("{:.2}", item.cost),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_grant_audit_views(items: &[LegacyGrantAudit]) -> Vec<LegacyGrantAuditView> {
    let mut views: Vec<LegacyGrantAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_grant_audit_view(item));
    }
    views
}

pub fn sum_legacy_grant_audit_view(view: &LegacyGrantAuditView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_cost + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_grant_audit("code-364", "name-404");
        let view = to_legacy_grant_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_grant_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
