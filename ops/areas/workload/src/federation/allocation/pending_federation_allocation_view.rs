use super::*;

use acme_kit::text;

/// How screens and exports show a pending federation allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationAllocationView {
    pub code_text: String,
    pub label: String,
    pub audience: String,
    pub cost: String,
    pub threshold: String,
    pub priority: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_pending_federation_allocation_view(item: &PendingFederationAllocation) -> PendingFederationAllocationView {
    PendingFederationAllocationView {
        code_text: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 80),
        audience: item.audience.clone(),
        cost: format!("{:.2}", item.cost),
        threshold: format!("{:.2}", item.threshold),
        priority: item.priority.to_string(),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_federation_allocation_views(
    items: &[PendingFederationAllocation],
) -> Vec<PendingFederationAllocationView> {
    let mut views: Vec<PendingFederationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_federation_allocation_view(item));
    }
    views
}

pub fn sum_pending_federation_allocation_view(view: &PendingFederationAllocationView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_cost + parsed_threshold + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_federation_allocation("code-831", "label-541");
        let view = to_pending_federation_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_federation_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
