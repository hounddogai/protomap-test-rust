use super::*;

/// How screens and exports show a plan target.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanTargetView {
    pub code: String,
    pub label: String,
    pub name: String,
    pub archived: String,
    pub ratio: String,
    pub attempts: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_plan_target_view(item: &PlanTarget) -> PlanTargetView {
    PlanTargetView {
        code: item.code.clone(),
        label: item.label.clone(),
        name: item.name.clone(),
        archived: format!("{}", item.archived),
        ratio: format!("{:.2}", item.ratio),
        attempts: item.attempts.to_string(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_target_views(items: &[PlanTarget]) -> Vec<PlanTargetView> {
    let mut views: Vec<PlanTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_target_view(item));
    }
    views
}

pub fn sum_plan_target_view(view: &PlanTargetView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_ratio + (parsed_attempts as f64) + (parsed_batch_size as f64))
}
