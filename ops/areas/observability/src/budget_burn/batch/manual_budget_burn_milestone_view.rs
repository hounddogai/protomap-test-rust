use super::*;

use acme_kit::text;

/// How screens and exports show a manual budget burn milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnMilestoneView {
    pub code_text: String,
    pub label: String,
    pub reviewer_text: String,
    pub tenant_id: String,
    pub priority: String,
    pub limit: String,
    pub namespace: String,
    pub version_text: String,
    pub status_label: String,
}

pub fn to_manual_budget_burn_milestone_view(item: &ManualBudgetBurnMilestone) -> ManualBudgetBurnMilestoneView {
    ManualBudgetBurnMilestoneView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        reviewer_text: text::truncate(&item.reviewer, 80),
        tenant_id: item.tenant_id.clone(),
        priority: item.priority.to_string(),
        limit: item.limit.to_string(),
        namespace: item.namespace.clone(),
        version_text: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_budget_burn_milestone_views(
    items: &[ManualBudgetBurnMilestone],
) -> Vec<ManualBudgetBurnMilestoneView> {
    let mut views: Vec<ManualBudgetBurnMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_budget_burn_milestone_view(item));
    }
    views
}

pub fn sum_manual_budget_burn_milestone_view(view: &ManualBudgetBurnMilestoneView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_limit as f64) + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_budget_burn_milestone("code-427", "label-672");
        let view = to_manual_budget_burn_milestone_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_budget_burn_milestone_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
