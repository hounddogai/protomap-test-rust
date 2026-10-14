use super::*;

use acme_kit::text;

/// How screens and exports show a rollback milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackMilestoneView {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub category: String,
    pub margin: String,
    pub cost_text: String,
    pub capacity: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_rollback_milestone_view(item: &RollbackMilestone) -> RollbackMilestoneView {
    RollbackMilestoneView {
        code: text::truncate(&item.code, 120),
        title: text::truncate(&item.title, 40),
        namespace: item.namespace.clone(),
        category: text::truncate(&item.category, 80),
        margin: format!("{:.2}", item.margin),
        cost_text: format!("{:.2}", item.cost),
        capacity: item.capacity.to_string(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_milestone_views(items: &[RollbackMilestone]) -> Vec<RollbackMilestoneView> {
    let mut views: Vec<RollbackMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_milestone_view(item));
    }
    views
}

pub fn sum_rollback_milestone_view(view: &RollbackMilestoneView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_margin + parsed_cost + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_milestone("code-94", "title-138");
        let view = to_rollback_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
