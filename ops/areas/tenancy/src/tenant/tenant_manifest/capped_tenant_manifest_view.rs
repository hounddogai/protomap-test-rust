use super::*;

use acme_kit::text;

/// How screens and exports show a capped tenant manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantManifestView {
    pub code: String,
    pub title: String,
    pub summary_text: String,
    pub grace_days: String,
    pub active: String,
    pub principal: String,
    pub approved: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_capped_tenant_manifest_view(item: &CappedTenantManifest) -> CappedTenantManifestView {
    CappedTenantManifestView {
        code: item.code.clone(),
        title: item.title.clone(),
        summary_text: item.summary.clone(),
        grace_days: item.grace_days.to_string(),
        active: format!("{}", item.active),
        principal: text::truncate(&item.principal, 80),
        approved: format!("{}", item.approved),
        currency_text: text::truncate(&item.currency, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_tenant_manifest_views(items: &[CappedTenantManifest]) -> Vec<CappedTenantManifestView> {
    let mut views: Vec<CappedTenantManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_tenant_manifest_view(item));
    }
    views
}

pub fn sum_capped_tenant_manifest_view(view: &CappedTenantManifestView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_tenant_manifest("code-338", "title-558");
        let view = to_capped_tenant_manifest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_tenant_manifest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
