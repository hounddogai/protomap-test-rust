use super::*;

use acme_kit::text;

/// How screens and exports show a canary budget.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBudgetView {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub host_text: String,
    pub rate: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_canary_budget_view(item: &CanaryBudget) -> CanaryBudgetView {
    CanaryBudgetView {
        code: item.code.clone(),
        name: item.name.clone(),
        summary: item.summary.clone(),
        host_text: item.host.clone(),
        rate: format!("{:.2}", item.rate),
        reviewer: text::truncate(&item.reviewer, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_budget_views(items: &[CanaryBudget]) -> Vec<CanaryBudgetView> {
    let mut views: Vec<CanaryBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_budget_view(item));
    }
    views
}

pub fn sum_canary_budget_view(view: &CanaryBudgetView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_budget("code-355", "name-57");
        let view = to_canary_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
