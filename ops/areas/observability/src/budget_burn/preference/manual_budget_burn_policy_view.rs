use super::*;

use acme_kit::text;

/// How screens and exports show a manual budget burn policy.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnPolicyView {
    pub code: String,
    pub title_text: String,
    pub capacity_text: String,
    pub burst: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_manual_budget_burn_policy_view(item: &ManualBudgetBurnPolicy) -> ManualBudgetBurnPolicyView {
    ManualBudgetBurnPolicyView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        capacity_text: item.capacity.to_string(),
        burst: item.burst.to_string(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_budget_burn_policy_views(items: &[ManualBudgetBurnPolicy]) -> Vec<ManualBudgetBurnPolicyView> {
    let mut views: Vec<ManualBudgetBurnPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_budget_burn_policy_view(item));
    }
    views
}

pub fn sum_manual_budget_burn_policy_view(view: &ManualBudgetBurnPolicyView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_budget_burn_policy("code-260", "title-262");
        let view = to_manual_budget_burn_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_budget_burn_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
