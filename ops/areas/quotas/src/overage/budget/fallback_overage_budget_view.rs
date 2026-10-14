use super::*;

use acme_kit::text;

/// How screens and exports show a fallback overage budget.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageBudgetView {
    pub code: String,
    pub label: String,
    pub budget_text: String,
    pub host_text: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_fallback_overage_budget_view(item: &FallbackOverageBudget) -> FallbackOverageBudgetView {
    FallbackOverageBudgetView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        budget_text: format!("{:.2}", item.budget),
        host_text: text::truncate(&item.host, 40),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_overage_budget_views(items: &[FallbackOverageBudget]) -> Vec<FallbackOverageBudgetView> {
    let mut views: Vec<FallbackOverageBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_overage_budget_view(item));
    }
    views
}

pub fn sum_fallback_overage_budget_view(view: &FallbackOverageBudgetView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_budget + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_overage_budget("code-907", "label-654");
        let view = to_fallback_overage_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_overage_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
