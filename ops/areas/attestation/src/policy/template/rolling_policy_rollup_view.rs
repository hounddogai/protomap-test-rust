use super::*;

use acme_kit::text;

/// How screens and exports show a rolling policy rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyRollupView {
    pub code: String,
    pub label_text: String,
    pub category_text: String,
    pub updated_at_text: String,
    pub title: String,
    pub limit_text: String,
    pub budget_text: String,
    pub ends_at_text: String,
    pub cost: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_rolling_policy_rollup_view(item: &RollingPolicyRollup) -> RollingPolicyRollupView {
    RollingPolicyRollupView {
        code: text::truncate(&item.code, 80),
        label_text: text::truncate(&item.label, 120),
        category_text: item.category.clone(),
        updated_at_text: item.updated_at.to_string(),
        title: text::truncate(&item.title, 64),
        limit_text: item.limit.to_string(),
        budget_text: format!("{:.2}", item.budget),
        ends_at_text: item.ends_at.to_string(),
        cost: format!("{:.2}", item.cost),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_policy_rollup_views(items: &[RollingPolicyRollup]) -> Vec<RollingPolicyRollupView> {
    let mut views: Vec<RollingPolicyRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_policy_rollup_view(item));
    }
    views
}

pub fn sum_rolling_policy_rollup_view(view: &RollingPolicyRollupView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_limit as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_policy_rollup("code-283", "label-167");
        let view = to_rolling_policy_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_policy_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
