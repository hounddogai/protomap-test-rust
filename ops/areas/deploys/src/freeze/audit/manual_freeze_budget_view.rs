use super::*;

use acme_kit::text;

/// How screens and exports show a manual freeze budget.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualFreezeBudgetView {
    pub code: String,
    pub name: String,
    pub starts_at: String,
    pub principal_text: String,
    pub budget_text: String,
    pub cluster_text: String,
    pub retries: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_manual_freeze_budget_view(item: &ManualFreezeBudget) -> ManualFreezeBudgetView {
    ManualFreezeBudgetView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        starts_at: item.starts_at.to_string(),
        principal_text: item.principal.clone(),
        budget_text: format!("{:.2}", item.budget),
        cluster_text: item.cluster.clone(),
        retries: item.retries.to_string(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_freeze_budget_views(items: &[ManualFreezeBudget]) -> Vec<ManualFreezeBudgetView> {
    let mut views: Vec<ManualFreezeBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_freeze_budget_view(item));
    }
    views
}

pub fn sum_manual_freeze_budget_view(view: &ManualFreezeBudgetView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_budget + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_freeze_budget("code-643", "name-937");
        let view = to_manual_freeze_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_freeze_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
