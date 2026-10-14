use super::*;

/// How screens and exports show a rolling budget burn preference.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnPreferenceView {
    pub code: String,
    pub name_text: String,
    pub amount: String,
    pub issuer: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_rolling_budget_burn_preference_view(item: &RollingBudgetBurnPreference) -> RollingBudgetBurnPreferenceView {
    RollingBudgetBurnPreferenceView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        issuer: item.issuer.clone(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_budget_burn_preference_views(
    items: &[RollingBudgetBurnPreference],
) -> Vec<RollingBudgetBurnPreferenceView> {
    let mut views: Vec<RollingBudgetBurnPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_budget_burn_preference_view(item));
    }
    views
}

pub fn sum_rolling_budget_burn_preference_view(view: &RollingBudgetBurnPreferenceView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_budget_burn_preference("code-363", "name-693");
        let view = to_rolling_budget_burn_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_budget_burn_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
