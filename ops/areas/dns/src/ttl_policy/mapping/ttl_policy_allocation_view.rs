use super::*;

use acme_kit::text;

/// How screens and exports show a ttl policy allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAllocationView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub threshold_text: String,
    pub region_text: String,
    pub name: String,
    pub principal_text: String,
    pub status_label: String,
}

pub fn to_ttl_policy_allocation_view(item: &TtlPolicyAllocation) -> TtlPolicyAllocationView {
    TtlPolicyAllocationView {
        code: item.code.clone(),
        label: item.label.clone(),
        currency: text::truncate(&item.currency, 64),
        threshold_text: format!("{:.2}", item.threshold),
        region_text: item.region.clone(),
        name: item.name.clone(),
        principal_text: text::truncate(&item.principal, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_allocation_views(items: &[TtlPolicyAllocation]) -> Vec<TtlPolicyAllocationView> {
    let mut views: Vec<TtlPolicyAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_allocation_view(item));
    }
    views
}

pub fn sum_ttl_policy_allocation_view(view: &TtlPolicyAllocationView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_ttl_policy_allocation("code-355", "label-565");
        let view = to_ttl_policy_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_ttl_policy_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
