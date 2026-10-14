use super::*;

use acme_kit::text;

/// How screens and exports show a migration audit.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationAuditView {
    pub code: String,
    pub label_text: String,
    pub summary_text: String,
    pub grace_days: String,
    pub owner_id: String,
    pub ttl_seconds_text: String,
    pub minimum: String,
    pub visible_text: String,
    pub status_label: String,
}

pub fn to_migration_audit_view(item: &MigrationAudit) -> MigrationAuditView {
    MigrationAuditView {
        code: text::truncate(&item.code, 64),
        label_text: text::truncate(&item.label, 40),
        summary_text: item.summary.clone(),
        grace_days: item.grace_days.to_string(),
        owner_id: item.owner_id.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        minimum: format!("{:.2}", item.minimum),
        visible_text: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_audit_views(items: &[MigrationAudit]) -> Vec<MigrationAuditView> {
    let mut views: Vec<MigrationAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_audit_view(item));
    }
    views
}

pub fn sum_migration_audit_view(view: &MigrationAuditView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_ttl_seconds as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_audit("code-220", "label-157");
        let view = to_migration_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
