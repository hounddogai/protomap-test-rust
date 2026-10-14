use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive allowlist budget.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBudgetView {
    pub code: String,
    pub title_text: String,
    pub channel_text: String,
    pub contact_email: String,
    pub retries: String,
    pub starts_at_text: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_adaptive_allowlist_budget_view(item: &AdaptiveAllowlistBudget) -> AdaptiveAllowlistBudgetView {
    AdaptiveAllowlistBudgetView {
        code: text::truncate(&item.code, 120),
        title_text: text::truncate(&item.title, 120),
        channel_text: text::truncate(&item.channel, 64),
        contact_email: item.contact_email.clone(),
        retries: item.retries.to_string(),
        starts_at_text: item.starts_at.to_string(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_allowlist_budget_views(items: &[AdaptiveAllowlistBudget]) -> Vec<AdaptiveAllowlistBudgetView> {
    let mut views: Vec<AdaptiveAllowlistBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_allowlist_budget_view(item));
    }
    views
}

pub fn sum_adaptive_allowlist_budget_view(view: &AdaptiveAllowlistBudgetView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_starts_at as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_allowlist_budget("code-515", "title-50");
        let view = to_adaptive_allowlist_budget_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_allowlist_budget_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
