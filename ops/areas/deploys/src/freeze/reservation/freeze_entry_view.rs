use super::*;

use acme_kit::text;

/// How screens and exports show a freeze entry.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeEntryView {
    pub code_text: String,
    pub label: String,
    pub unit: String,
    pub maximum_text: String,
    pub approved: String,
    pub contact_email_text: String,
    pub created_at: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_freeze_entry_view(item: &FreezeEntry) -> FreezeEntryView {
    FreezeEntryView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        unit: item.unit.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        approved: format!("{}", item.approved),
        contact_email_text: text::truncate(&item.contact_email, 40),
        created_at: item.created_at.to_string(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_entry_views(items: &[FreezeEntry]) -> Vec<FreezeEntryView> {
    let mut views: Vec<FreezeEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_entry_view(item));
    }
    views
}

pub fn sum_freeze_entry_view(view: &FreezeEntryView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_maximum + (parsed_created_at as f64))
}
