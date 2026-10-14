use super::*;

use acme_kit::text;

/// How screens and exports show a grant plan.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantPlanView {
    pub code: String,
    pub title: String,
    pub burst: String,
    pub unit: String,
    pub floor_value_text: String,
    pub ceiling_text: String,
    pub scope: String,
    pub namespace: String,
    pub minimum: String,
    pub name: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_grant_plan_view(item: &GrantPlan) -> GrantPlanView {
    GrantPlanView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        burst: item.burst.to_string(),
        unit: item.unit.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        ceiling_text: format!("{:.2}", item.ceiling),
        scope: item.scope.clone(),
        namespace: text::truncate(&item.namespace, 40),
        minimum: format!("{:.2}", item.minimum),
        name: text::truncate(&item.name, 80),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_plan_views(items: &[GrantPlan]) -> Vec<GrantPlanView> {
    let mut views: Vec<GrantPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_plan_view(item));
    }
    views
}

pub fn sum_grant_plan_view(view: &GrantPlanView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_burst as f64 + parsed_floor_value + parsed_ceiling)
}
