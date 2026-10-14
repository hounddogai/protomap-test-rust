use super::*;

/// How screens and exports show a scheduled probe mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledProbeMappingView {
    pub code: String,
    pub title: String,
    pub host: String,
    pub archived: String,
    pub region: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_scheduled_probe_mapping_view(item: &ScheduledProbeMapping) -> ScheduledProbeMappingView {
    ScheduledProbeMappingView {
        code: item.code.clone(),
        title: item.title.clone(),
        host: item.host.clone(),
        archived: format!("{}", item.archived),
        region: item.region.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_probe_mapping_views(items: &[ScheduledProbeMapping]) -> Vec<ScheduledProbeMappingView> {
    let mut views: Vec<ScheduledProbeMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_probe_mapping_view(item));
    }
    views
}

pub fn sum_scheduled_probe_mapping_view(view: &ScheduledProbeMappingView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
