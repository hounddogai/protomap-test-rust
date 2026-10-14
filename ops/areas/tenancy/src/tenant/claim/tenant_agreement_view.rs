use super::*;

use acme_kit::text;

/// How screens and exports show a tenant agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantAgreementView {
    pub code_text: String,
    pub title: String,
    pub notes_text: String,
    pub scope_text: String,
    pub burst: String,
    pub threshold: String,
    pub created_at: String,
    pub name_text: String,
    pub budget: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_tenant_agreement_view(item: &TenantAgreement) -> TenantAgreementView {
    TenantAgreementView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        notes_text: text::truncate(&item.notes, 40),
        scope_text: text::truncate(&item.scope, 80),
        burst: item.burst.to_string(),
        threshold: format!("{:.2}", item.threshold),
        created_at: item.created_at.to_string(),
        name_text: item.name.clone(),
        budget: format!("{:.2}", item.budget),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_agreement_views(items: &[TenantAgreement]) -> Vec<TenantAgreementView> {
    let mut views: Vec<TenantAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_agreement_view(item));
    }
    views
}

pub fn sum_tenant_agreement_view(view: &TenantAgreementView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_burst as f64 + parsed_threshold + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tenant_agreement("code-138", "title-402");
        let view = to_tenant_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tenant_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
