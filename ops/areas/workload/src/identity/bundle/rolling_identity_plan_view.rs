use super::*;

use acme_kit::text;

/// How screens and exports show a rolling identity plan.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityPlanView {
    pub code: String,
    pub title_text: String,
    pub grace_days_text: String,
    pub principal: String,
    pub revision: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_rolling_identity_plan_view(item: &RollingIdentityPlan) -> RollingIdentityPlanView {
    RollingIdentityPlanView {
        code: text::truncate(&item.code, 40),
        title_text: item.title.clone(),
        grace_days_text: item.grace_days.to_string(),
        principal: text::truncate(&item.principal, 64),
        revision: item.revision.to_string(),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_identity_plan_views(items: &[RollingIdentityPlan]) -> Vec<RollingIdentityPlanView> {
    let mut views: Vec<RollingIdentityPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_identity_plan_view(item));
    }
    views
}

pub fn sum_rolling_identity_plan_view(view: &RollingIdentityPlanView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_revision as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_identity_plan("code-549", "title-834");
        let view = to_rolling_identity_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_identity_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
