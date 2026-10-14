use super::*;

/// How screens and exports show a alert plan.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertPlanView {
    pub code: String,
    pub title_text: String,
    pub quantity: String,
    pub discount_text: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_alert_plan_view(item: &AlertPlan) -> AlertPlanView {
    AlertPlanView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        quantity: item.quantity.to_string(),
        discount_text: format!("{:.2}", item.discount),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_plan_views(items: &[AlertPlan]) -> Vec<AlertPlanView> {
    let mut views: Vec<AlertPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_plan_view(item));
    }
    views
}

pub fn sum_alert_plan_view(view: &AlertPlanView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_discount + parsed_score)
}
