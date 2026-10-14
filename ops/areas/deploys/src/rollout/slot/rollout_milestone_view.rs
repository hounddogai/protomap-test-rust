use super::*;

use acme_kit::text;

/// How screens and exports show a rollout milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutMilestoneView {
    pub code: String,
    pub title: String,
    pub ceiling: String,
    pub external_ref: String,
    pub rotation_days: String,
    pub quota_units: String,
    pub floor_value: String,
    pub principal_text: String,
    pub duration_minutes: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_rollout_milestone_view(item: &RolloutMilestone) -> RolloutMilestoneView {
    RolloutMilestoneView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        external_ref: item.external_ref.clone(),
        rotation_days: item.rotation_days.to_string(),
        quota_units: item.quota_units.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        principal_text: item.principal.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_milestone_views(items: &[RolloutMilestone]) -> Vec<RolloutMilestoneView> {
    let mut views: Vec<RolloutMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_milestone_view(item));
    }
    views
}

pub fn sum_rollout_milestone_view(view: &RolloutMilestoneView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ceiling + (parsed_rotation_days as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_milestone("code-829", "title-824");
        let view = to_rollout_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
