use super::*;

use acme_kit::text;

/// How screens and exports show a approval allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAllocationView {
    pub code_text: String,
    pub title_text: String,
    pub host: String,
    pub owner_id: String,
    pub archived: String,
    pub tenant_id_text: String,
    pub reviewer_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_approval_allocation_view(item: &ApprovalAllocation) -> ApprovalAllocationView {
    ApprovalAllocationView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        host: text::truncate(&item.host, 80),
        owner_id: item.owner_id.clone(),
        archived: format!("{}", item.archived),
        tenant_id_text: item.tenant_id.clone(),
        reviewer_text: item.reviewer.clone(),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_allocation_views(items: &[ApprovalAllocation]) -> Vec<ApprovalAllocationView> {
    let mut views: Vec<ApprovalAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_allocation_view(item));
    }
    views
}

pub fn sum_approval_allocation_view(view: &ApprovalAllocationView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_approval_allocation("code-906", "title-56");
        let view = to_approval_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_approval_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
