use super::*;

use acme_kit::text;

/// How screens and exports show a global egress allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressAllocationView {
    pub code: String,
    pub name: String,
    pub host_text: String,
    pub contact_email: String,
    pub owner_id: String,
    pub created_at_text: String,
    pub phone: String,
    pub summary: String,
    pub sequence: String,
    pub country: String,
    pub status_label: String,
}

pub fn to_global_egress_allocation_view(item: &GlobalEgressAllocation) -> GlobalEgressAllocationView {
    GlobalEgressAllocationView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        host_text: item.host.clone(),
        contact_email: item.contact_email.clone(),
        owner_id: item.owner_id.clone(),
        created_at_text: item.created_at.to_string(),
        phone: text::truncate(&item.phone, 64),
        summary: item.summary.clone(),
        sequence: item.sequence.to_string(),
        country: item.country.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_egress_allocation_views(items: &[GlobalEgressAllocation]) -> Vec<GlobalEgressAllocationView> {
    let mut views: Vec<GlobalEgressAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_egress_allocation_view(item));
    }
    views
}

pub fn sum_global_egress_allocation_view(view: &GlobalEgressAllocationView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_egress_allocation("code-371", "name-358");
        let view = to_global_egress_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_egress_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
