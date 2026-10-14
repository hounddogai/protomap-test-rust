use super::*;

use acme_kit::text;

/// How screens and exports show a legacy tenant audit.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTenantAuditView {
    pub code: String,
    pub title_text: String,
    pub score: String,
    pub external_ref: String,
    pub segment_text: String,
    pub revision_text: String,
    pub unit_text: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_legacy_tenant_audit_view(item: &LegacyTenantAudit) -> LegacyTenantAuditView {
    LegacyTenantAuditView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        score: format!("{:.2}", item.score),
        external_ref: text::truncate(&item.external_ref, 120),
        segment_text: item.segment.clone(),
        revision_text: item.revision.to_string(),
        unit_text: text::truncate(&item.unit, 64),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_tenant_audit_views(items: &[LegacyTenantAudit]) -> Vec<LegacyTenantAuditView> {
    let mut views: Vec<LegacyTenantAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_tenant_audit_view(item));
    }
    views
}

pub fn sum_legacy_tenant_audit_view(view: &LegacyTenantAuditView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_score + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_tenant_audit("code-46", "title-249");
        let view = to_legacy_tenant_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_tenant_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
