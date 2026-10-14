use super::*;

use acme_kit::text;

/// How screens and exports show a legacy migration audit.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMigrationAuditView {
    pub code: String,
    pub label: String,
    pub discount: String,
    pub ends_at: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_legacy_migration_audit_view(item: &LegacyMigrationAudit) -> LegacyMigrationAuditView {
    LegacyMigrationAuditView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        discount: format!("{:.2}", item.discount),
        ends_at: item.ends_at.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_migration_audit_views(items: &[LegacyMigrationAudit]) -> Vec<LegacyMigrationAuditView> {
    let mut views: Vec<LegacyMigrationAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_migration_audit_view(item));
    }
    views
}

pub fn sum_legacy_migration_audit_view(view: &LegacyMigrationAuditView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_discount + (parsed_ends_at as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_migration_audit("code-112", "label-276");
        let view = to_legacy_migration_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_migration_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
