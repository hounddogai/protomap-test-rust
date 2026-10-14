use super::*;

use acme_kit::text;

/// How screens and exports show a usage budget.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageBudgetView {
    pub code: String,
    pub label_text: String,
    pub rotation_days: String,
    pub scope_text: String,
    pub revision: String,
    pub audience_text: String,
    pub cluster: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_usage_budget_view(item: &UsageBudget) -> UsageBudgetView {
    UsageBudgetView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        rotation_days: item.rotation_days.to_string(),
        scope_text: text::truncate(&item.scope, 40),
        revision: item.revision.to_string(),
        audience_text: item.audience.clone(),
        cluster: item.cluster.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_budget_views(items: &[UsageBudget]) -> Vec<UsageBudgetView> {
    let mut views: Vec<UsageBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_budget_view(item));
    }
    views
}

pub fn sum_usage_budget_view(view: &UsageBudgetView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_revision as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_budget("code-44", "label-261");
        let view = to_usage_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
