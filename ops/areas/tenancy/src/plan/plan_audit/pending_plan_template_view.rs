use super::*;

use acme_kit::text;

/// How screens and exports show a pending plan template.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanTemplateView {
    pub code: String,
    pub name: String,
    pub batch_size_text: String,
    pub currency: String,
    pub rotation_days: String,
    pub weight_text: String,
    pub created_at: String,
    pub floor_value: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_pending_plan_template_view(item: &PendingPlanTemplate) -> PendingPlanTemplateView {
    PendingPlanTemplateView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        batch_size_text: item.batch_size.to_string(),
        currency: text::truncate(&item.currency, 64),
        rotation_days: item.rotation_days.to_string(),
        weight_text: format!("{:.2}", item.weight),
        created_at: item.created_at.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        fingerprint: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_plan_template_views(items: &[PendingPlanTemplate]) -> Vec<PendingPlanTemplateView> {
    let mut views: Vec<PendingPlanTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_plan_template_view(item));
    }
    views
}

pub fn sum_pending_plan_template_view(view: &PendingPlanTemplateView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_rotation_days as f64) + parsed_weight)
}
