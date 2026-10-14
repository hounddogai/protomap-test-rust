use super::*;

use acme_kit::text;

/// How screens and exports show a weighted escalation tier.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationTierView {
    pub code_text: String,
    pub title_text: String,
    pub notes: String,
    pub cost: String,
    pub weight: String,
    pub quantity: String,
    pub region: String,
    pub source: String,
    pub active: String,
    pub name_text: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_weighted_escalation_tier_view(item: &WeightedEscalationTier) -> WeightedEscalationTierView {
    WeightedEscalationTierView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        notes: item.notes.clone(),
        cost: format!("{:.2}", item.cost),
        weight: format!("{:.2}", item.weight),
        quantity: item.quantity.to_string(),
        region: text::truncate(&item.region, 120),
        source: text::truncate(&item.source, 120),
        active: format!("{}", item.active),
        name_text: item.name.clone(),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_escalation_tier_views(items: &[WeightedEscalationTier]) -> Vec<WeightedEscalationTierView> {
    let mut views: Vec<WeightedEscalationTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_escalation_tier_view(item));
    }
    views
}

pub fn sum_weighted_escalation_tier_view(view: &WeightedEscalationTierView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_cost + parsed_weight + (parsed_quantity as f64))
}
