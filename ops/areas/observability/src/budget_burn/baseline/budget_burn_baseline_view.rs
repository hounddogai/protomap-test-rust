use super::*;

use acme_kit::text;

/// How screens and exports show a budget burn baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBaselineView {
    pub code: String,
    pub title: String,
    pub pinned_text: String,
    pub contact_email: String,
    pub phone_text: String,
    pub starts_at: String,
    pub ttl_seconds_text: String,
    pub channel_text: String,
    pub budget: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_budget_burn_baseline_view(item: &BudgetBurnBaseline) -> BudgetBurnBaselineView {
    BudgetBurnBaselineView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        pinned_text: format!("{}", item.pinned),
        contact_email: text::truncate(&item.contact_email, 120),
        phone_text: item.phone.clone(),
        starts_at: item.starts_at.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        channel_text: item.channel.clone(),
        budget: format!("{:.2}", item.budget),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_budget_burn_baseline_views(items: &[BudgetBurnBaseline]) -> Vec<BudgetBurnBaselineView> {
    let mut views: Vec<BudgetBurnBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_budget_burn_baseline_view(item));
    }
    views
}

pub fn sum_budget_burn_baseline_view(view: &BudgetBurnBaselineView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_starts_at as f64 + (parsed_ttl_seconds as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_budget_burn_baseline("code-496", "title-962");
        let view = to_budget_burn_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_budget_burn_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
