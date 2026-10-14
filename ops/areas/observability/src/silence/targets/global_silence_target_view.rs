use super::*;

/// How screens and exports show a global silence target.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceTargetView {
    pub code_text: String,
    pub name_text: String,
    pub quantity: String,
    pub summary: String,
    pub capacity_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_global_silence_target_view(item: &GlobalSilenceTarget) -> GlobalSilenceTargetView {
    GlobalSilenceTargetView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        quantity: item.quantity.to_string(),
        summary: item.summary.clone(),
        capacity_text: item.capacity.to_string(),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_silence_target_views(items: &[GlobalSilenceTarget]) -> Vec<GlobalSilenceTargetView> {
    let mut views: Vec<GlobalSilenceTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_silence_target_view(item));
    }
    views
}

pub fn sum_global_silence_target_view(view: &GlobalSilenceTargetView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_capacity as f64))
}
