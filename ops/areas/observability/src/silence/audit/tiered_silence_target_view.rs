use super::*;

/// How screens and exports show a tiered silence target.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceTargetView {
    pub code_text: String,
    pub name: String,
    pub amount: String,
    pub description_text: String,
    pub active: String,
    pub approved: String,
    pub unit_text: String,
    pub title: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_tiered_silence_target_view(item: &TieredSilenceTarget) -> TieredSilenceTargetView {
    TieredSilenceTargetView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        description_text: item.description.clone(),
        active: format!("{}", item.active),
        approved: format!("{}", item.approved),
        unit_text: item.unit.clone(),
        title: item.title.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_silence_target_views(items: &[TieredSilenceTarget]) -> Vec<TieredSilenceTargetView> {
    let mut views: Vec<TieredSilenceTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_silence_target_view(item));
    }
    views
}

pub fn sum_tiered_silence_target_view(view: &TieredSilenceTargetView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
