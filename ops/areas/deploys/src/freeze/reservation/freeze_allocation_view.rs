use super::*;

use acme_kit::text;

/// How screens and exports show a freeze allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAllocationView {
    pub code: String,
    pub name: String,
    pub ends_at: String,
    pub revision: String,
    pub capacity: String,
    pub summary: String,
    pub ttl_seconds: String,
    pub unit: String,
    pub sequence_text: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_freeze_allocation_view(item: &FreezeAllocation) -> FreezeAllocationView {
    FreezeAllocationView {
        code: item.code.clone(),
        name: item.name.clone(),
        ends_at: item.ends_at.to_string(),
        revision: item.revision.to_string(),
        capacity: item.capacity.to_string(),
        summary: text::truncate(&item.summary, 80),
        ttl_seconds: item.ttl_seconds.to_string(),
        unit: text::truncate(&item.unit, 80),
        sequence_text: item.sequence.to_string(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_allocation_views(items: &[FreezeAllocation]) -> Vec<FreezeAllocationView> {
    let mut views: Vec<FreezeAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_allocation_view(item));
    }
    views
}

pub fn sum_freeze_allocation_view(view: &FreezeAllocationView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_revision as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_allocation("code-72", "name-553");
        let view = to_freeze_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
