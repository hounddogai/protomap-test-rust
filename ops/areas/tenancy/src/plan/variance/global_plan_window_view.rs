use super::*;

/// How screens and exports show a global plan window.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanWindowView {
    pub code: String,
    pub title_text: String,
    pub weight: String,
    pub limit_text: String,
    pub margin: String,
    pub principal_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_global_plan_window_view(item: &GlobalPlanWindow) -> GlobalPlanWindowView {
    GlobalPlanWindowView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        limit_text: item.limit.to_string(),
        margin: format!("{:.2}", item.margin),
        principal_text: item.principal.clone(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_plan_window_views(items: &[GlobalPlanWindow]) -> Vec<GlobalPlanWindowView> {
    let mut views: Vec<GlobalPlanWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_plan_window_view(item));
    }
    views
}

pub fn sum_global_plan_window_view(view: &GlobalPlanWindowView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_weight + (parsed_limit as f64) + parsed_margin)
}
