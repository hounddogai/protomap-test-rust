use super::*;

/// How screens and exports show a default peering adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPeeringAdjustmentView {
    pub code: String,
    pub label: String,
    pub score: String,
    pub quantity_text: String,
    pub ceiling: String,
    pub floor_value: String,
    pub retries_text: String,
    pub region: String,
    pub fingerprint: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_default_peering_adjustment_view(item: &DefaultPeeringAdjustment) -> DefaultPeeringAdjustmentView {
    DefaultPeeringAdjustmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        score: format!("{:.2}", item.score),
        quantity_text: item.quantity.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        floor_value: format!("{:.2}", item.floor_value),
        retries_text: item.retries.to_string(),
        region: item.region.clone(),
        fingerprint: item.fingerprint.clone(),
        name: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_peering_adjustment_views(items: &[DefaultPeeringAdjustment]) -> Vec<DefaultPeeringAdjustmentView> {
    let mut views: Vec<DefaultPeeringAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_peering_adjustment_view(item));
    }
    views
}

pub fn sum_default_peering_adjustment_view(view: &DefaultPeeringAdjustmentView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_score + (parsed_quantity as f64) + parsed_ceiling)
}
