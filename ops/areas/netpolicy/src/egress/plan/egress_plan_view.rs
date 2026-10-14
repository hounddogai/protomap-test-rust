use super::*;

use acme_kit::text;

/// How screens and exports show a egress plan.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressPlanView {
    pub code: String,
    pub title: String,
    pub floor_value: String,
    pub created_at_text: String,
    pub name: String,
    pub currency: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_egress_plan_view(item: &EgressPlan) -> EgressPlanView {
    EgressPlanView {
        code: item.code.clone(),
        title: item.title.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        created_at_text: item.created_at.to_string(),
        name: text::truncate(&item.name, 40),
        currency: text::truncate(&item.currency, 120),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_plan_views(items: &[EgressPlan]) -> Vec<EgressPlanView> {
    let mut views: Vec<EgressPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_plan_view(item));
    }
    views
}

pub fn sum_egress_plan_view(view: &EgressPlanView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_floor_value + (parsed_created_at as f64))
}
