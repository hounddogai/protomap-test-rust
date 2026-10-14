use super::*;

/// How screens and exports show a sink plan.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkPlanView {
    pub code: String,
    pub title: String,
    pub discount_text: String,
    pub host: String,
    pub retries: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_sink_plan_view(item: &SinkPlan) -> SinkPlanView {
    SinkPlanView {
        code: item.code.clone(),
        title: item.title.clone(),
        discount_text: format!("{:.2}", item.discount),
        host: item.host.clone(),
        retries: item.retries.to_string(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_sink_plan_views(items: &[SinkPlan]) -> Vec<SinkPlanView> {
    let mut views: Vec<SinkPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_sink_plan_view(item));
    }
    views
}

pub fn sum_sink_plan_view(view: &SinkPlanView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_discount + (parsed_retries as f64) + parsed_cost)
}
