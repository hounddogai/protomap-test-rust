use super::*;

use acme_kit::text;

/// How screens and exports show a primary rule budget.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleBudgetView {
    pub code_text: String,
    pub name_text: String,
    pub margin: String,
    pub grace_days: String,
    pub ratio: String,
    pub window_days: String,
    pub approved: String,
    pub floor_value: String,
    pub capacity: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_primary_rule_budget_view(item: &PrimaryRuleBudget) -> PrimaryRuleBudgetView {
    PrimaryRuleBudgetView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        margin: format!("{:.2}", item.margin),
        grace_days: item.grace_days.to_string(),
        ratio: format!("{:.2}", item.ratio),
        window_days: item.window_days.to_string(),
        approved: format!("{}", item.approved),
        floor_value: format!("{:.2}", item.floor_value),
        capacity: item.capacity.to_string(),
        country: text::truncate(&item.country, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_rule_budget_views(items: &[PrimaryRuleBudget]) -> Vec<PrimaryRuleBudgetView> {
    let mut views: Vec<PrimaryRuleBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_rule_budget_view(item));
    }
    views
}

pub fn sum_primary_rule_budget_view(view: &PrimaryRuleBudgetView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_margin + (parsed_grace_days as f64) + parsed_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_rule_budget("code-643", "name-299");
        let view = to_primary_rule_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_rule_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
