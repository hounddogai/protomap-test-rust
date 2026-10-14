use super::*;

use acme_kit::text;

/// How screens and exports show a rolling identity milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityMilestoneView {
    pub code: String,
    pub name: String,
    pub ends_at: String,
    pub amount: String,
    pub starts_at: String,
    pub updated_at: String,
    pub label_text: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_rolling_identity_milestone_view(item: &RollingIdentityMilestone) -> RollingIdentityMilestoneView {
    RollingIdentityMilestoneView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        ends_at: item.ends_at.to_string(),
        amount: format!("{:.2}", item.amount),
        starts_at: item.starts_at.to_string(),
        updated_at: item.updated_at.to_string(),
        label_text: item.label.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_identity_milestone_views(items: &[RollingIdentityMilestone]) -> Vec<RollingIdentityMilestoneView> {
    let mut views: Vec<RollingIdentityMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_identity_milestone_view(item));
    }
    views
}

pub fn sum_rolling_identity_milestone_view(view: &RollingIdentityMilestoneView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_amount + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_identity_milestone("code-4", "name-760");
        let view = to_rolling_identity_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_identity_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
