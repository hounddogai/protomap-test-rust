use super::*;

use acme_kit::text;

/// How screens and exports show a rollout checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutCheckpointView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub budget: String,
    pub source: String,
    pub ends_at: String,
    pub starts_at: String,
    pub reviewer: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_rollout_checkpoint_view(item: &RolloutCheckpoint) -> RolloutCheckpointView {
    RolloutCheckpointView {
        code: text::truncate(&item.code, 120),
        label: item.label.clone(),
        cost: format!("{:.2}", item.cost),
        budget: format!("{:.2}", item.budget),
        source: item.source.clone(),
        ends_at: item.ends_at.to_string(),
        starts_at: item.starts_at.to_string(),
        reviewer: text::truncate(&item.reviewer, 120),
        category_text: item.category.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_checkpoint_views(items: &[RolloutCheckpoint]) -> Vec<RolloutCheckpointView> {
    let mut views: Vec<RolloutCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_checkpoint_view(item));
    }
    views
}

pub fn sum_rollout_checkpoint_view(view: &RolloutCheckpointView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_cost + parsed_budget + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_checkpoint("code-645", "label-743");
        let view = to_rollout_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
