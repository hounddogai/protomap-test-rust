use super::*;

use acme_kit::text;

/// How screens and exports show a page profile.
#[derive(Debug, Clone, PartialEq)]
pub struct PageProfileView {
    pub code: String,
    pub label: String,
    pub rotation_days_text: String,
    pub cluster: String,
    pub active: String,
    pub reviewer: String,
    pub principal_text: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_page_profile_view(item: &PageProfile) -> PageProfileView {
    PageProfileView {
        code: text::truncate(&item.code, 40),
        label: item.label.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        cluster: item.cluster.clone(),
        active: format!("{}", item.active),
        reviewer: item.reviewer.clone(),
        principal_text: item.principal.clone(),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_profile_views(items: &[PageProfile]) -> Vec<PageProfileView> {
    let mut views: Vec<PageProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_profile_view(item));
    }
    views
}

pub fn sum_page_profile_view(view: &PageProfileView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_rotation_days as f64 + parsed_discount)
}
