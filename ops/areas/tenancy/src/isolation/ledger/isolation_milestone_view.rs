use super::*;

use acme_kit::text;

/// How screens and exports show a isolation milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMilestoneView {
    pub code_text: String,
    pub label: String,
    pub scope: String,
    pub revision: String,
    pub region: String,
    pub grace_days: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_isolation_milestone_view(item: &IsolationMilestone) -> IsolationMilestoneView {
    IsolationMilestoneView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        scope: item.scope.clone(),
        revision: item.revision.to_string(),
        region: text::truncate(&item.region, 64),
        grace_days: item.grace_days.to_string(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_milestone_views(items: &[IsolationMilestone]) -> Vec<IsolationMilestoneView> {
    let mut views: Vec<IsolationMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_milestone_view(item));
    }
    views
}

pub fn sum_isolation_milestone_view(view: &IsolationMilestoneView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_grace_days as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_isolation_milestone("code-855", "label-8");
        let view = to_isolation_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_isolation_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
