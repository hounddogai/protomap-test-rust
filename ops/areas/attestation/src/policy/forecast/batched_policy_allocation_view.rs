use super::*;

use acme_kit::text;

/// How screens and exports show a batched policy allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAllocationView {
    pub code: String,
    pub label_text: String,
    pub capacity: String,
    pub phone_text: String,
    pub fingerprint_text: String,
    pub revision_text: String,
    pub reviewer_text: String,
    pub status_label: String,
}

pub fn to_batched_policy_allocation_view(item: &BatchedPolicyAllocation) -> BatchedPolicyAllocationView {
    BatchedPolicyAllocationView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        capacity: item.capacity.to_string(),
        phone_text: item.phone.clone(),
        fingerprint_text: item.fingerprint.clone(),
        revision_text: item.revision.to_string(),
        reviewer_text: text::truncate(&item.reviewer, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_policy_allocation_views(items: &[BatchedPolicyAllocation]) -> Vec<BatchedPolicyAllocationView> {
    let mut views: Vec<BatchedPolicyAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_policy_allocation_view(item));
    }
    views
}

pub fn sum_batched_policy_allocation_view(view: &BatchedPolicyAllocationView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_policy_allocation("code-41", "label-361");
        let view = to_batched_policy_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_policy_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
