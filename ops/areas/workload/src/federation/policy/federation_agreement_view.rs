use super::*;

use acme_kit::text;

/// How screens and exports show a federation agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationAgreementView {
    pub code: String,
    pub name_text: String,
    pub category: String,
    pub minimum: String,
    pub cipher: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_federation_agreement_view(item: &FederationAgreement) -> FederationAgreementView {
    FederationAgreementView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        category: text::truncate(&item.category, 40),
        minimum: format!("{:.2}", item.minimum),
        cipher: text::truncate(&item.cipher, 40),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_agreement_views(items: &[FederationAgreement]) -> Vec<FederationAgreementView> {
    let mut views: Vec<FederationAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_agreement_view(item));
    }
    views
}

pub fn sum_federation_agreement_view(view: &FederationAgreementView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_minimum + parsed_discount)
}
