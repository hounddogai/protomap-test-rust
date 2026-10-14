use super::*;

use acme_kit::text;

/// How screens and exports show a regional escalation adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalEscalationAdjustmentView {
    pub code: String,
    pub title_text: String,
    pub external_ref: String,
    pub budget: String,
    pub segment_text: String,
    pub ttl_seconds: String,
    pub currency_text: String,
    pub updated_at: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_regional_escalation_adjustment_view(item: &RegionalEscalationAdjustment) -> RegionalEscalationAdjustmentView {
    RegionalEscalationAdjustmentView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        external_ref: text::truncate(&item.external_ref, 80),
        budget: format!("{:.2}", item.budget),
        segment_text: text::truncate(&item.segment, 120),
        ttl_seconds: item.ttl_seconds.to_string(),
        currency_text: item.currency.clone(),
        updated_at: item.updated_at.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_escalation_adjustment_views(
    items: &[RegionalEscalationAdjustment],
) -> Vec<RegionalEscalationAdjustmentView> {
    let mut views: Vec<RegionalEscalationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_escalation_adjustment_view(item));
    }
    views
}

pub fn sum_regional_escalation_adjustment_view(view: &RegionalEscalationAdjustmentView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_budget + (parsed_ttl_seconds as f64) + (parsed_updated_at as f64))
}
