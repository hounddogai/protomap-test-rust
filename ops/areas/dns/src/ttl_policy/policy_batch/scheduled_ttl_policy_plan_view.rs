use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled ttl policy plan.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyPlanView {
    pub code_text: String,
    pub title: String,
    pub audience: String,
    pub host: String,
    pub quantity_text: String,
    pub summary_text: String,
    pub sequence: String,
    pub expires_at_text: String,
    pub quota_units: String,
    pub key_id: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_scheduled_ttl_policy_plan_view(item: &ScheduledTtlPolicyPlan) -> ScheduledTtlPolicyPlanView {
    ScheduledTtlPolicyPlanView {
        code_text: text::truncate(&item.code, 120),
        title: item.title.clone(),
        audience: item.audience.clone(),
        host: item.host.clone(),
        quantity_text: item.quantity.to_string(),
        summary_text: item.summary.clone(),
        sequence: item.sequence.to_string(),
        expires_at_text: item.expires_at.to_string(),
        quota_units: item.quota_units.to_string(),
        key_id: text::truncate(&item.key_id, 40),
        principal: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_ttl_policy_plan_views(items: &[ScheduledTtlPolicyPlan]) -> Vec<ScheduledTtlPolicyPlanView> {
    let mut views: Vec<ScheduledTtlPolicyPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_ttl_policy_plan_view(item));
    }
    views
}

pub fn sum_scheduled_ttl_policy_plan_view(view: &ScheduledTtlPolicyPlanView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_sequence as f64) + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_ttl_policy_plan("code-52", "title-69");
        let view = to_scheduled_ttl_policy_plan_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_ttl_policy_plan_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
