use super::*;

use acme_kit::text;

/// How screens and exports show a global federation notice.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationNoticeView {
    pub code_text: String,
    pub label: String,
    pub capacity_text: String,
    pub budget: String,
    pub visible: String,
    pub segment_text: String,
    pub notes: String,
    pub discount: String,
    pub issuer: String,
    pub scope_text: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_global_federation_notice_view(item: &GlobalFederationNotice) -> GlobalFederationNoticeView {
    GlobalFederationNoticeView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        capacity_text: item.capacity.to_string(),
        budget: format!("{:.2}", item.budget),
        visible: format!("{}", item.visible),
        segment_text: text::truncate(&item.segment, 120),
        notes: item.notes.clone(),
        discount: format!("{:.2}", item.discount),
        issuer: item.issuer.clone(),
        scope_text: text::truncate(&item.scope, 40),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_federation_notice_views(items: &[GlobalFederationNotice]) -> Vec<GlobalFederationNoticeView> {
    let mut views: Vec<GlobalFederationNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_federation_notice_view(item));
    }
    views
}

pub fn sum_global_federation_notice_view(view: &GlobalFederationNoticeView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_budget + parsed_discount)
}
