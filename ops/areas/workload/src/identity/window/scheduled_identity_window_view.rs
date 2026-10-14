use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled identity window.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityWindowView {
    pub code_text: String,
    pub title_text: String,
    pub name: String,
    pub scope: String,
    pub principal_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_scheduled_identity_window_view(item: &ScheduledIdentityWindow) -> ScheduledIdentityWindowView {
    ScheduledIdentityWindowView {
        code_text: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        name: item.name.clone(),
        scope: item.scope.clone(),
        principal_text: item.principal.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_identity_window_views(items: &[ScheduledIdentityWindow]) -> Vec<ScheduledIdentityWindowView> {
    let mut views: Vec<ScheduledIdentityWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_identity_window_view(item));
    }
    views
}

pub fn sum_scheduled_identity_window_view(view: &ScheduledIdentityWindowView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
