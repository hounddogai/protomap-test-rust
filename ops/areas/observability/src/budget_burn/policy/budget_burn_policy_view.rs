use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn policy.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnPolicyView {
    pub code: String,
    pub name_text: String,
    pub tenant_id: String,
    pub batch_size_text: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_budget_burn_policy_view(item: &BudgetBurnPolicy) -> BudgetBurnPolicyView {
    BudgetBurnPolicyView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        tenant_id: text::truncate(&item.tenant_id, 40),
        batch_size_text: item.batch_size.to_string(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_policy_views(items: &[BudgetBurnPolicy]) -> Vec<BudgetBurnPolicyView> {
    let mut views: Vec<BudgetBurnPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_policy_view(item));
    }
    views
}

pub fn sum_budget_burn_policy_view(view: &BudgetBurnPolicyView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_policy("code-803", "name-502");
        let view = to_budget_burn_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
