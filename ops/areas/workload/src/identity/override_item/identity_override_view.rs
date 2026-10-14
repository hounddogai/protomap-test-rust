use super::*;

use acme_kit::text;

/// How screens and exports show a identity override.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityOverrideView {
    pub code_text: String,
    pub name: String,
    pub created_at_text: String,
    pub retries: String,
    pub visible_text: String,
    pub revision: String,
    pub label: String,
    pub batch_size: String,
    pub margin_text: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_identity_override_view(item: &IdentityOverride) -> IdentityOverrideView {
    IdentityOverrideView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        created_at_text: item.created_at.to_string(),
        retries: item.retries.to_string(),
        visible_text: format!("{}", item.visible),
        revision: item.revision.to_string(),
        label: item.label.clone(),
        batch_size: item.batch_size.to_string(),
        margin_text: format!("{:.2}", item.margin),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_override_views(items: &[IdentityOverride]) -> Vec<IdentityOverrideView> {
    let mut views: Vec<IdentityOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_override_view(item));
    }
    views
}

pub fn sum_identity_override_view(view: &IdentityOverrideView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_retries as f64) + (parsed_revision as f64))
}
