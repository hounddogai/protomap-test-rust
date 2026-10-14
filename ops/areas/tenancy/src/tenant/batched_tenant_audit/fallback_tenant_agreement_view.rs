use super::*;

use acme_kit::text;

/// How screens and exports show a fallback tenant agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTenantAgreementView {
    pub code_text: String,
    pub label_text: String,
    pub margin: String,
    pub cipher: String,
    pub timezone_text: String,
    pub currency: String,
    pub visible_text: String,
    pub fingerprint_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_fallback_tenant_agreement_view(item: &FallbackTenantAgreement) -> FallbackTenantAgreementView {
    FallbackTenantAgreementView {
        code_text: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        cipher: item.cipher.clone(),
        timezone_text: item.timezone.clone(),
        currency: item.currency.clone(),
        visible_text: format!("{}", item.visible),
        fingerprint_text: item.fingerprint.clone(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_tenant_agreement_views(items: &[FallbackTenantAgreement]) -> Vec<FallbackTenantAgreementView> {
    let mut views: Vec<FallbackTenantAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_tenant_agreement_view(item));
    }
    views
}

pub fn sum_fallback_tenant_agreement_view(view: &FallbackTenantAgreementView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_tenant_agreement("code-424", "label-773");
        let view = to_fallback_tenant_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_tenant_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
