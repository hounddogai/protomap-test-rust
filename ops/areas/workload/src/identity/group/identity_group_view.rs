use super::*;

/// How screens and exports show a identity group.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityGroupView {
    pub code_text: String,
    pub name_text: String,
    pub margin: String,
    pub starts_at: String,
    pub priority_text: String,
    pub visible: String,
    pub audience_text: String,
    pub status_label: String,
}

pub fn to_identity_group_view(item: &IdentityGroup) -> IdentityGroupView {
    IdentityGroupView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        starts_at: item.starts_at.to_string(),
        priority_text: item.priority.to_string(),
        visible: format!("{}", item.visible),
        audience_text: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_group_views(items: &[IdentityGroup]) -> Vec<IdentityGroupView> {
    let mut views: Vec<IdentityGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_group_view(item));
    }
    views
}

pub fn sum_identity_group_view(view: &IdentityGroupView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_margin + (parsed_starts_at as f64) + (parsed_priority as f64))
}
