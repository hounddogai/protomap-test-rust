use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAgreementView {
    pub code_text: String,
    pub title: String,
    pub audience: String,
    pub description_text: String,
    pub approved: String,
    pub key_id: String,
    pub unit: String,
    pub window_days: String,
    pub tenant_id: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_postmortem_agreement_view(item: &PostmortemAgreement) -> PostmortemAgreementView {
    PostmortemAgreementView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        audience: text::truncate(&item.audience, 80),
        description_text: item.description.clone(),
        approved: format!("{}", item.approved),
        key_id: text::truncate(&item.key_id, 40),
        unit: text::truncate(&item.unit, 64),
        window_days: item.window_days.to_string(),
        tenant_id: text::truncate(&item.tenant_id, 40),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_agreement_views(items: &[PostmortemAgreement]) -> Vec<PostmortemAgreementView> {
    let mut views: Vec<PostmortemAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_agreement_view(item));
    }
    views
}

pub fn sum_postmortem_agreement_view(view: &PostmortemAgreementView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_created_at as f64))
}
