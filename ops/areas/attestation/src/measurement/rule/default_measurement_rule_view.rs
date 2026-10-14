use super::*;

/// How screens and exports show a default measurement rule.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementRuleView {
    pub code_text: String,
    pub title: String,
    pub ratio: String,
    pub duration_minutes_text: String,
    pub revision: String,
    pub window_days_text: String,
    pub region_text: String,
    pub notes: String,
    pub pinned: String,
    pub ceiling_text: String,
    pub sequence: String,
    pub status_label: String,
}

pub fn to_default_measurement_rule_view(item: &DefaultMeasurementRule) -> DefaultMeasurementRuleView {
    DefaultMeasurementRuleView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        ratio: format!("{:.2}", item.ratio),
        duration_minutes_text: item.duration_minutes.to_string(),
        revision: item.revision.to_string(),
        window_days_text: item.window_days.to_string(),
        region_text: item.region.clone(),
        notes: item.notes.clone(),
        pinned: format!("{}", item.pinned),
        ceiling_text: format!("{:.2}", item.ceiling),
        sequence: item.sequence.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_measurement_rule_views(items: &[DefaultMeasurementRule]) -> Vec<DefaultMeasurementRuleView> {
    let mut views: Vec<DefaultMeasurementRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_measurement_rule_view(item));
    }
    views
}

pub fn sum_default_measurement_rule_view(view: &DefaultMeasurementRuleView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_ratio + (parsed_duration_minutes as f64) + (parsed_revision as f64))
}
