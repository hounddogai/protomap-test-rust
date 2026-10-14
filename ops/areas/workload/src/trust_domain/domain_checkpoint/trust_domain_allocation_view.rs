use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAllocationView {
    pub code_text: String,
    pub name: String,
    pub batch_size: String,
    pub owner_id_text: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_trust_domain_allocation_view(item: &TrustDomainAllocation) -> TrustDomainAllocationView {
    TrustDomainAllocationView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        batch_size: item.batch_size.to_string(),
        owner_id_text: item.owner_id.clone(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_allocation_views(items: &[TrustDomainAllocation]) -> Vec<TrustDomainAllocationView> {
    let mut views: Vec<TrustDomainAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_allocation_view(item));
    }
    views
}

pub fn sum_trust_domain_allocation_view(view: &TrustDomainAllocationView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_batch_size as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_allocation("code-223", "name-45");
        let view = to_trust_domain_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
