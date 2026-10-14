use super::*;

/// How screens and exports show a adaptive federation cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationCycleView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub channel: String,
    pub limit: String,
    pub timezone: String,
    pub owner_id: String,
    pub ceiling: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_adaptive_federation_cycle_view(item: &AdaptiveFederationCycle) -> AdaptiveFederationCycleView {
    AdaptiveFederationCycleView {
        code: item.code.clone(),
        label: item.label.clone(),
        cost: format!("{:.2}", item.cost),
        channel: item.channel.clone(),
        limit: item.limit.to_string(),
        timezone: item.timezone.clone(),
        owner_id: item.owner_id.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_federation_cycle_views(items: &[AdaptiveFederationCycle]) -> Vec<AdaptiveFederationCycleView> {
    let mut views: Vec<AdaptiveFederationCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_federation_cycle_view(item));
    }
    views
}

pub fn sum_adaptive_federation_cycle_view(view: &AdaptiveFederationCycleView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_cost + (parsed_limit as f64) + parsed_ceiling)
}
