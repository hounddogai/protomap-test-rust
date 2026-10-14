use super::*;

use acme_kit::text;

/// How screens and exports show a plan checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanCheckpointView {
    pub code: String,
    pub label: String,
    pub category_text: String,
    pub cost: String,
    pub rotation_days: String,
    pub ratio: String,
    pub status_label: String,
}

pub fn to_plan_checkpoint_view(item: &PlanCheckpoint) -> PlanCheckpointView {
    PlanCheckpointView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        category_text: text::truncate(&item.category, 120),
        cost: format!("{:.2}", item.cost),
        rotation_days: item.rotation_days.to_string(),
        ratio: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_checkpoint_views(items: &[PlanCheckpoint]) -> Vec<PlanCheckpointView> {
    let mut views: Vec<PlanCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_checkpoint_view(item));
    }
    views
}

pub fn sum_plan_checkpoint_view(view: &PlanCheckpointView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_cost + (parsed_rotation_days as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_plan_checkpoint("code-742", "label-9");
        let view = to_plan_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_plan_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
