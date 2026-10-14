use super::*;

use acme_kit::text;

/// How screens and exports show a fallback rollback policy.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRollbackPolicyView {
    pub code: String,
    pub title: String,
    pub region: String,
    pub reviewer: String,
    pub quota_units_text: String,
    pub budget: String,
    pub phone: String,
    pub created_at: String,
    pub timezone: String,
    pub status_label: String,
}

pub fn to_fallback_rollback_policy_view(item: &FallbackRollbackPolicy) -> FallbackRollbackPolicyView {
    FallbackRollbackPolicyView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        region: text::truncate(&item.region, 64),
        reviewer: item.reviewer.clone(),
        quota_units_text: item.quota_units.to_string(),
        budget: format!("{:.2}", item.budget),
        phone: text::truncate(&item.phone, 40),
        created_at: item.created_at.to_string(),
        timezone: text::truncate(&item.timezone, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_rollback_policy_views(items: &[FallbackRollbackPolicy]) -> Vec<FallbackRollbackPolicyView> {
    let mut views: Vec<FallbackRollbackPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_rollback_policy_view(item));
    }
    views
}

pub fn sum_fallback_rollback_policy_view(view: &FallbackRollbackPolicyView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_budget + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_rollback_policy("code-3", "title-281");
        let view = to_fallback_rollback_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_rollback_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
