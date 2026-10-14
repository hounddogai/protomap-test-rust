use super::*;

use acme_kit::text;

/// How screens and exports show a binding window.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingWindowView {
    pub code: String,
    pub title: String,
    pub principal_text: String,
    pub contact_email_text: String,
    pub grace_days_text: String,
    pub status_label: String,
}

pub fn to_binding_window_view(item: &BindingWindow) -> BindingWindowView {
    BindingWindowView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        principal_text: text::truncate(&item.principal, 40),
        contact_email_text: item.contact_email.clone(),
        grace_days_text: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_window_views(items: &[BindingWindow]) -> Vec<BindingWindowView> {
    let mut views: Vec<BindingWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_window_view(item));
    }
    views
}

pub fn sum_binding_window_view(view: &BindingWindowView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}
