use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn rule.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnRuleView {
    pub code: String,
    pub name_text: String,
    pub starts_at: String,
    pub issuer: String,
    pub rotation_days_text: String,
    pub pinned: String,
    pub country: String,
    pub timezone: String,
    pub region: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_budget_burn_rule_view(item: &BudgetBurnRule) -> BudgetBurnRuleView {
    BudgetBurnRuleView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        starts_at: item.starts_at.to_string(),
        issuer: item.issuer.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        pinned: format!("{}", item.pinned),
        country: item.country.clone(),
        timezone: item.timezone.clone(),
        region: text::truncate(&item.region, 80),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_rule_views(items: &[BudgetBurnRule]) -> Vec<BudgetBurnRuleView> {
    let mut views: Vec<BudgetBurnRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_rule_view(item));
    }
    views
}

pub fn sum_budget_burn_rule_view(view: &BudgetBurnRuleView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_rotation_days as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_rule("code-596", "name-786");
        let view = to_budget_burn_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
