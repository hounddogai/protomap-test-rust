use super::*;

use acme_kit::text;

/// How screens and exports show a peering allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAllocationView {
    pub code_text: String,
    pub name: String,
    pub label: String,
    pub reviewer: String,
    pub updated_at: String,
    pub batch_size: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_peering_allocation_view(item: &PeeringAllocation) -> PeeringAllocationView {
    PeeringAllocationView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        label: text::truncate(&item.label, 80),
        reviewer: item.reviewer.clone(),
        updated_at: item.updated_at.to_string(),
        batch_size: item.batch_size.to_string(),
        notes: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_peering_allocation_views(items: &[PeeringAllocation]) -> Vec<PeeringAllocationView> {
    let mut views: Vec<PeeringAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_peering_allocation_view(item));
    }
    views
}

pub fn sum_peering_allocation_view(view: &PeeringAllocationView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_batch_size as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_peering_allocation("code-43", "name-632");
        let view = to_peering_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_peering_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
