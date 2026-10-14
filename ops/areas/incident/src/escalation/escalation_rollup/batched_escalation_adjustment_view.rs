use super::*;

use acme_kit::text;

/// How screens and exports show a batched escalation adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEscalationAdjustmentView {
    pub code: String,
    pub name: String,
    pub owner_id_text: String,
    pub score: String,
    pub limit: String,
    pub approved: String,
    pub unit_text: String,
    pub title_text: String,
    pub capacity: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_batched_escalation_adjustment_view(item: &BatchedEscalationAdjustment) -> BatchedEscalationAdjustmentView {
    BatchedEscalationAdjustmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        owner_id_text: text::truncate(&item.owner_id, 64),
        score: format!("{:.2}", item.score),
        limit: item.limit.to_string(),
        approved: format!("{}", item.approved),
        unit_text: item.unit.clone(),
        title_text: item.title.clone(),
        capacity: item.capacity.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_escalation_adjustment_views(
    items: &[BatchedEscalationAdjustment],
) -> Vec<BatchedEscalationAdjustmentView> {
    let mut views: Vec<BatchedEscalationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_escalation_adjustment_view(item));
    }
    views
}

pub fn sum_batched_escalation_adjustment_view(view: &BatchedEscalationAdjustmentView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_score + (parsed_limit as f64) + (parsed_capacity as f64))
}
