use super::*;

/// How screens and exports show a regional timeline cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTimelineCycleView {
    pub code: String,
    pub title: String,
    pub quantity: String,
    pub region_text: String,
    pub revision: String,
    pub amount_text: String,
    pub checksum: String,
    pub phone_text: String,
    pub status_label: String,
}

pub fn to_regional_timeline_cycle_view(item: &RegionalTimelineCycle) -> RegionalTimelineCycleView {
    RegionalTimelineCycleView {
        code: item.code.clone(),
        title: item.title.clone(),
        quantity: item.quantity.to_string(),
        region_text: item.region.clone(),
        revision: item.revision.to_string(),
        amount_text: format!("{:.2}", item.amount),
        checksum: item.checksum.clone(),
        phone_text: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_timeline_cycle_views(items: &[RegionalTimelineCycle]) -> Vec<RegionalTimelineCycleView> {
    let mut views: Vec<RegionalTimelineCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_timeline_cycle_view(item));
    }
    views
}

pub fn sum_regional_timeline_cycle_view(view: &RegionalTimelineCycleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_revision as f64) + parsed_amount)
}
