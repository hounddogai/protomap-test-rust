use super::*;

use acme_kit::text;

/// How screens and exports show a lenient egress agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressAgreementView {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub timezone_text: String,
    pub region: String,
    pub external_ref: String,
    pub checksum_text: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_lenient_egress_agreement_view(item: &LenientEgressAgreement) -> LenientEgressAgreementView {
    LenientEgressAgreementView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        owner_id: item.owner_id.clone(),
        timezone_text: item.timezone.clone(),
        region: text::truncate(&item.region, 40),
        external_ref: item.external_ref.clone(),
        checksum_text: item.checksum.clone(),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_egress_agreement_views(items: &[LenientEgressAgreement]) -> Vec<LenientEgressAgreementView> {
    let mut views: Vec<LenientEgressAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_egress_agreement_view(item));
    }
    views
}

pub fn sum_lenient_egress_agreement_view(view: &LenientEgressAgreementView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
