use super::*;

use acme_kit::text;

/// How screens and exports show a policy milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMilestoneView {
    pub code: String,
    pub label: String,
    pub visible_text: String,
    pub quota_units_text: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_policy_milestone_view(item: &PolicyMilestone) -> PolicyMilestoneView {
    PolicyMilestoneView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        visible_text: format!("{}", item.visible),
        quota_units_text: item.quota_units.to_string(),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_milestone_views(items: &[PolicyMilestone]) -> Vec<PolicyMilestoneView> {
    let mut views: Vec<PolicyMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_milestone_view(item));
    }
    views
}

pub fn sum_policy_milestone_view(view: &PolicyMilestoneView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_milestone("code-512", "label-667");
        let view = to_policy_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
