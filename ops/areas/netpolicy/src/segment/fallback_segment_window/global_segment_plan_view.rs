use super::*;

use acme_kit::text;

/// How screens and exports show a global segment plan.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPlanView {
    pub code: String,
    pub label: String,
    pub unit_text: String,
    pub source: String,
    pub margin: String,
    pub floor_value_text: String,
    pub ends_at: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_global_segment_plan_view(item: &GlobalSegmentPlan) -> GlobalSegmentPlanView {
    GlobalSegmentPlanView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        unit_text: item.unit.clone(),
        source: item.source.clone(),
        margin: format!("{:.2}", item.margin),
        floor_value_text: format!("{:.2}", item.floor_value),
        ends_at: item.ends_at.to_string(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_segment_plan_views(items: &[GlobalSegmentPlan]) -> Vec<GlobalSegmentPlanView> {
    let mut views: Vec<GlobalSegmentPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_segment_plan_view(item));
    }
    views
}

pub fn sum_global_segment_plan_view(view: &GlobalSegmentPlanView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_margin + parsed_floor_value + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_segment_plan("code-824", "label-192");
        let view = to_global_segment_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_segment_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
