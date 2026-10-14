use super::*;

use acme_kit::text;

/// How screens and exports show a page group.
#[derive(Debug, Clone, PartialEq)]
pub struct PageGroupView {
    pub code: String,
    pub name_text: String,
    pub margin: String,
    pub currency: String,
    pub ratio: String,
    pub duration_minutes: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_page_group_view(item: &PageGroup) -> PageGroupView {
    PageGroupView {
        code: text::truncate(&item.code, 64),
        name_text: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        currency: text::truncate(&item.currency, 120),
        ratio: format!("{:.2}", item.ratio),
        duration_minutes: item.duration_minutes.to_string(),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_group_views(items: &[PageGroup]) -> Vec<PageGroupView> {
    let mut views: Vec<PageGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_group_view(item));
    }
    views
}

pub fn sum_page_group_view(view: &PageGroupView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_margin + parsed_ratio + (parsed_duration_minutes as f64))
}
