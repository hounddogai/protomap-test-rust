use super::*;

use acme_kit::text;

/// How screens and exports show a delegation budget.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBudgetView {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub region_text: String,
    pub segment: String,
    pub ends_at: String,
    pub locale: String,
    pub attempts: String,
    pub channel_text: String,
    pub status_label: String,
}

pub fn to_delegation_budget_view(item: &DelegationBudget) -> DelegationBudgetView {
    DelegationBudgetView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        fingerprint: item.fingerprint.clone(),
        region_text: item.region.clone(),
        segment: item.segment.clone(),
        ends_at: item.ends_at.to_string(),
        locale: item.locale.clone(),
        attempts: item.attempts.to_string(),
        channel_text: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_budget_views(items: &[DelegationBudget]) -> Vec<DelegationBudgetView> {
    let mut views: Vec<DelegationBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_budget_view(item));
    }
    views
}

pub fn sum_delegation_budget_view(view: &DelegationBudgetView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_budget("code-526", "name-383");
        let view = to_delegation_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
