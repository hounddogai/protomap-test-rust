use super::*;

use acme_kit::text;

/// How screens and exports show a pending limit plan.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPlanView {
    pub code: String,
    pub title_text: String,
    pub key_id: String,
    pub quota_units: String,
    pub ends_at: String,
    pub tenant_id_text: String,
    pub priority: String,
    pub description: String,
    pub scope_text: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_pending_limit_plan_view(item: &PendingLimitPlan) -> PendingLimitPlanView {
    PendingLimitPlanView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        key_id: item.key_id.clone(),
        quota_units: item.quota_units.to_string(),
        ends_at: item.ends_at.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        priority: item.priority.to_string(),
        description: item.description.clone(),
        scope_text: item.scope.clone(),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_limit_plan_views(items: &[PendingLimitPlan]) -> Vec<PendingLimitPlanView> {
    let mut views: Vec<PendingLimitPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_limit_plan_view(item));
    }
    views
}

pub fn sum_pending_limit_plan_view(view: &PendingLimitPlanView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_ends_at as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_limit_plan("code-753", "title-125");
        let view = to_pending_limit_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_limit_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
