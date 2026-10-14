use super::*;

use acme_kit::text;

/// How screens and exports show a tiered verifier allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierAllocationView {
    pub code: String,
    pub name: String,
    pub priority_text: String,
    pub sequence: String,
    pub created_at: String,
    pub audience: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_tiered_verifier_allocation_view(item: &TieredVerifierAllocation) -> TieredVerifierAllocationView {
    TieredVerifierAllocationView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        priority_text: item.priority.to_string(),
        sequence: item.sequence.to_string(),
        created_at: item.created_at.to_string(),
        audience: item.audience.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_verifier_allocation_views(items: &[TieredVerifierAllocation]) -> Vec<TieredVerifierAllocationView> {
    let mut views: Vec<TieredVerifierAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_verifier_allocation_view(item));
    }
    views
}

pub fn sum_tiered_verifier_allocation_view(view: &TieredVerifierAllocationView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_sequence as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_verifier_allocation("code-85", "name-518");
        let view = to_tiered_verifier_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_verifier_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
