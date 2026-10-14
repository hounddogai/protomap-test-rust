use super::*;

use acme_kit::text;

/// How screens and exports show a lenient tenant audit.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTenantAuditView {
    pub code_text: String,
    pub title: String,
    pub reviewer_text: String,
    pub amount: String,
    pub score: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_lenient_tenant_audit_view(item: &LenientTenantAudit) -> LenientTenantAuditView {
    LenientTenantAuditView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        reviewer_text: text::truncate(&item.reviewer, 64),
        amount: format!("{:.2}", item.amount),
        score: format!("{:.2}", item.score),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_tenant_audit_views(items: &[LenientTenantAudit]) -> Vec<LenientTenantAuditView> {
    let mut views: Vec<LenientTenantAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_tenant_audit_view(item));
    }
    views
}

pub fn sum_lenient_tenant_audit_view(view: &LenientTenantAuditView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_amount + parsed_score + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_tenant_audit("code-365", "title-913");
        let view = to_lenient_tenant_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_tenant_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
