use super::*;

use acme_kit::text;

/// How screens and exports show a approval agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAgreementView {
    pub code_text: String,
    pub title_text: String,
    pub capacity: String,
    pub country_text: String,
    pub batch_size_text: String,
    pub name: String,
    pub principal_text: String,
    pub contact_email_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_approval_agreement_view(item: &ApprovalAgreement) -> ApprovalAgreementView {
    ApprovalAgreementView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        capacity: item.capacity.to_string(),
        country_text: text::truncate(&item.country, 64),
        batch_size_text: item.batch_size.to_string(),
        name: item.name.clone(),
        principal_text: item.principal.clone(),
        contact_email_text: text::truncate(&item.contact_email, 64),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_agreement_views(items: &[ApprovalAgreement]) -> Vec<ApprovalAgreementView> {
    let mut views: Vec<ApprovalAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_agreement_view(item));
    }
    views
}

pub fn sum_approval_agreement_view(view: &ApprovalAgreementView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_batch_size as f64) + parsed_score)
}
