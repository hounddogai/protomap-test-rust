use super::*;

/// How screens and exports show a scheduled egress profile.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressProfileView {
    pub code: String,
    pub title: String,
    pub country_text: String,
    pub revision: String,
    pub channel: String,
    pub quantity_text: String,
    pub margin: String,
    pub active_text: String,
    pub weight: String,
    pub status_label: String,
}

pub fn to_scheduled_egress_profile_view(item: &ScheduledEgressProfile) -> ScheduledEgressProfileView {
    ScheduledEgressProfileView {
        code: item.code.clone(),
        title: item.title.clone(),
        country_text: item.country.clone(),
        revision: item.revision.to_string(),
        channel: item.channel.clone(),
        quantity_text: item.quantity.to_string(),
        margin: format!("{:.2}", item.margin),
        active_text: format!("{}", item.active),
        weight: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_egress_profile_views(items: &[ScheduledEgressProfile]) -> Vec<ScheduledEgressProfileView> {
    let mut views: Vec<ScheduledEgressProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_egress_profile_view(item));
    }
    views
}

pub fn sum_scheduled_egress_profile_view(view: &ScheduledEgressProfileView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_quantity as f64) + parsed_margin)
}
