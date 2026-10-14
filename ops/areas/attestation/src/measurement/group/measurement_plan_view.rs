use super::*;

/// How screens and exports show a measurement plan.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPlanView {
    pub code_text: String,
    pub title_text: String,
    pub priority: String,
    pub threshold: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_measurement_plan_view(item: &MeasurementPlan) -> MeasurementPlanView {
    MeasurementPlanView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        priority: item.priority.to_string(),
        threshold: format!("{:.2}", item.threshold),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_plan_views(items: &[MeasurementPlan]) -> Vec<MeasurementPlanView> {
    let mut views: Vec<MeasurementPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_plan_view(item));
    }
    views
}

pub fn sum_measurement_plan_view(view: &MeasurementPlanView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_priority as f64 + parsed_threshold)
}
