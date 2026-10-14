use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy plan.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyPlanView {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub grace_days_text: String,
    pub segment_text: String,
    pub margin_text: String,
    pub principal_text: String,
    pub rate: String,
    pub archived_text: String,
    pub priority: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_ttl_policy_plan_view(item: &TtlPolicyPlan) -> TtlPolicyPlanView {
    TtlPolicyPlanView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        locale: text::truncate(&item.locale, 80),
        grace_days_text: item.grace_days.to_string(),
        segment_text: text::truncate(&item.segment, 64),
        margin_text: format!("{:.2}", item.margin),
        principal_text: text::truncate(&item.principal, 64),
        rate: format!("{:.2}", item.rate),
        archived_text: format!("{}", item.archived),
        priority: item.priority.to_string(),
        cluster: text::truncate(&item.cluster, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_plan_views(items: &[TtlPolicyPlan]) -> Vec<TtlPolicyPlanView> {
    let mut views: Vec<TtlPolicyPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_plan_view(item));
    }
    views
}

pub fn sum_ttl_policy_plan_view(view: &TtlPolicyPlanView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_margin + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_plan("code-88", "label-794");
        let view = to_ttl_policy_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
