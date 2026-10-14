use super::*;

/// How screens and exports show a scheduled budget burn preference.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledBudgetBurnPreferenceView {
    pub code_text: String,
    pub label_text: String,
    pub grace_days_text: String,
    pub priority: String,
    pub updated_at: String,
    pub threshold_text: String,
    pub contact_email: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_scheduled_budget_burn_preference_view(
    item: &ScheduledBudgetBurnPreference,
) -> ScheduledBudgetBurnPreferenceView {
    ScheduledBudgetBurnPreferenceView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        grace_days_text: item.grace_days.to_string(),
        priority: item.priority.to_string(),
        updated_at: item.updated_at.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        contact_email: item.contact_email.clone(),
        currency_text: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_budget_burn_preference_views(
    items: &[ScheduledBudgetBurnPreference],
) -> Vec<ScheduledBudgetBurnPreferenceView> {
    let mut views: Vec<ScheduledBudgetBurnPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_budget_burn_preference_view(item));
    }
    views
}

pub fn sum_scheduled_budget_burn_preference_view(view: &ScheduledBudgetBurnPreferenceView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_priority as f64) + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_budget_burn_preference("code-208", "label-92");
        let view = to_scheduled_budget_burn_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_budget_burn_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
