use super::*;

use acme_kit::text;

/// How screens and exports show a primary budget burn rule.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnRuleView {
    pub code_text: String,
    pub title: String,
    pub namespace_text: String,
    pub description_text: String,
    pub active: String,
    pub rate: String,
    pub weight_text: String,
    pub locale: String,
    pub version_text: String,
    pub status_label: String,
}

pub fn to_primary_budget_burn_rule_view(item: &PrimaryBudgetBurnRule) -> PrimaryBudgetBurnRuleView {
    PrimaryBudgetBurnRuleView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        namespace_text: text::truncate(&item.namespace, 40),
        description_text: item.description.clone(),
        active: format!("{}", item.active),
        rate: format!("{:.2}", item.rate),
        weight_text: format!("{:.2}", item.weight),
        locale: item.locale.clone(),
        version_text: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_budget_burn_rule_views(items: &[PrimaryBudgetBurnRule]) -> Vec<PrimaryBudgetBurnRuleView> {
    let mut views: Vec<PrimaryBudgetBurnRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_budget_burn_rule_view(item));
    }
    views
}

pub fn sum_primary_budget_burn_rule_view(view: &PrimaryBudgetBurnRuleView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_rate + parsed_weight + (parsed_version as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_budget_burn_rule("code-728", "title-643");
        let view = to_primary_budget_burn_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_budget_burn_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
