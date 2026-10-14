use super::*;

use acme_kit::text;

/// How screens and exports show a legacy policy agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyAgreementView {
    pub code: String,
    pub title: String,
    pub category: String,
    pub external_ref: String,
    pub amount: String,
    pub created_at: String,
    pub channel_text: String,
    pub margin: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_legacy_policy_agreement_view(item: &LegacyPolicyAgreement) -> LegacyPolicyAgreementView {
    LegacyPolicyAgreementView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        category: item.category.clone(),
        external_ref: text::truncate(&item.external_ref, 40),
        amount: format!("{:.2}", item.amount),
        created_at: item.created_at.to_string(),
        channel_text: item.channel.clone(),
        margin: format!("{:.2}", item.margin),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_policy_agreement_views(items: &[LegacyPolicyAgreement]) -> Vec<LegacyPolicyAgreementView> {
    let mut views: Vec<LegacyPolicyAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_policy_agreement_view(item));
    }
    views
}

pub fn sum_legacy_policy_agreement_view(view: &LegacyPolicyAgreementView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_amount + (parsed_created_at as f64) + parsed_margin)
}
