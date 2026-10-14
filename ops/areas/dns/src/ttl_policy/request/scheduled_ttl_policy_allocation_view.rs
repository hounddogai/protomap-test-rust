use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled ttl policy allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTtlPolicyAllocationView {
    pub code_text: String,
    pub label: String,
    pub budget: String,
    pub active: String,
    pub pinned_text: String,
    pub updated_at: String,
    pub locale: String,
    pub phone: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_scheduled_ttl_policy_allocation_view(
    item: &ScheduledTtlPolicyAllocation,
) -> ScheduledTtlPolicyAllocationView {
    ScheduledTtlPolicyAllocationView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        budget: format!("{:.2}", item.budget),
        active: format!("{}", item.active),
        pinned_text: format!("{}", item.pinned),
        updated_at: item.updated_at.to_string(),
        locale: text::truncate(&item.locale, 64),
        phone: item.phone.clone(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_ttl_policy_allocation_views(
    items: &[ScheduledTtlPolicyAllocation],
) -> Vec<ScheduledTtlPolicyAllocationView> {
    let mut views: Vec<ScheduledTtlPolicyAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_ttl_policy_allocation_view(item));
    }
    views
}

pub fn sum_scheduled_ttl_policy_allocation_view(view: &ScheduledTtlPolicyAllocationView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_budget + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_ttl_policy_allocation("code-647", "label-739");
        let view = to_scheduled_ttl_policy_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_ttl_policy_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
