use super::*;

use acme_kit::text;

/// How screens and exports show a regional canary allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanaryAllocationView {
    pub code: String,
    pub name: String,
    pub minimum_text: String,
    pub channel: String,
    pub scope_text: String,
    pub visible: String,
    pub maximum: String,
    pub priority: String,
    pub burst_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_regional_canary_allocation_view(item: &RegionalCanaryAllocation) -> RegionalCanaryAllocationView {
    RegionalCanaryAllocationView {
        code: item.code.clone(),
        name: item.name.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        channel: text::truncate(&item.channel, 120),
        scope_text: text::truncate(&item.scope, 40),
        visible: format!("{}", item.visible),
        maximum: format!("{:.2}", item.maximum),
        priority: item.priority.to_string(),
        burst_text: item.burst.to_string(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_canary_allocation_views(items: &[RegionalCanaryAllocation]) -> Vec<RegionalCanaryAllocationView> {
    let mut views: Vec<RegionalCanaryAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_canary_allocation_view(item));
    }
    views
}

pub fn sum_regional_canary_allocation_view(view: &RegionalCanaryAllocationView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_minimum + parsed_maximum + (parsed_priority as f64))
}
