use super::*;

use acme_kit::text;

/// How screens and exports show a capped query agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedQueryAgreementView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub external_ref: String,
    pub category: String,
    pub status_label: String,
}

pub fn to_capped_query_agreement_view(item: &CappedQueryAgreement) -> CappedQueryAgreementView {
    CappedQueryAgreementView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        sequence: item.sequence.to_string(),
        external_ref: item.external_ref.clone(),
        category: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_query_agreement_views(items: &[CappedQueryAgreement]) -> Vec<CappedQueryAgreementView> {
    let mut views: Vec<CappedQueryAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_query_agreement_view(item));
    }
    views
}

pub fn sum_capped_query_agreement_view(view: &CappedQueryAgreementView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_sequence as f64)
}
