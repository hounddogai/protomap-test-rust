use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist budget.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBudgetView {
    pub code: String,
    pub name_text: String,
    pub burst: String,
    pub description_text: String,
    pub limit_text: String,
    pub grace_days_text: String,
    pub threshold: String,
    pub external_ref: String,
    pub reviewer: String,
    pub owner_id: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_allowlist_budget_view(item: &AllowlistBudget) -> AllowlistBudgetView {
    AllowlistBudgetView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        burst: item.burst.to_string(),
        description_text: text::truncate(&item.description, 120),
        limit_text: item.limit.to_string(),
        grace_days_text: item.grace_days.to_string(),
        threshold: format!("{:.2}", item.threshold),
        external_ref: item.external_ref.clone(),
        reviewer: item.reviewer.clone(),
        owner_id: item.owner_id.clone(),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_budget_views(items: &[AllowlistBudget]) -> Vec<AllowlistBudgetView> {
    let mut views: Vec<AllowlistBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_budget_view(item));
    }
    views
}

pub fn sum_allowlist_budget_view(view: &AllowlistBudgetView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_limit as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_budget("code-856", "name-319");
        let view = to_allowlist_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
