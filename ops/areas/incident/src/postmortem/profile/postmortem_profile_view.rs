use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem profile.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProfileView {
    pub code_text: String,
    pub name: String,
    pub label: String,
    pub capacity: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_postmortem_profile_view(item: &PostmortemProfile) -> PostmortemProfileView {
    PostmortemProfileView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        label: text::truncate(&item.label, 40),
        capacity: item.capacity.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_profile_views(items: &[PostmortemProfile]) -> Vec<PostmortemProfileView> {
    let mut views: Vec<PostmortemProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_profile_view(item));
    }
    views
}

pub fn sum_postmortem_profile_view(view: &PostmortemProfileView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_duration_minutes as f64))
}
