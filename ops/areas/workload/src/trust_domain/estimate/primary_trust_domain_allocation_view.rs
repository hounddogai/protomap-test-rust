use super::*;

use acme_kit::text;

/// How screens and exports show a primary trust domain allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainAllocationView {
    pub code: String,
    pub name: String,
    pub active_text: String,
    pub source: String,
    pub tenant_id_text: String,
    pub checksum: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_primary_trust_domain_allocation_view(
    item: &PrimaryTrustDomainAllocation,
) -> PrimaryTrustDomainAllocationView {
    PrimaryTrustDomainAllocationView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        active_text: format!("{}", item.active),
        source: item.source.clone(),
        tenant_id_text: item.tenant_id.clone(),
        checksum: item.checksum.clone(),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_trust_domain_allocation_views(
    items: &[PrimaryTrustDomainAllocation],
) -> Vec<PrimaryTrustDomainAllocationView> {
    let mut views: Vec<PrimaryTrustDomainAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_trust_domain_allocation_view(item));
    }
    views
}

pub fn sum_primary_trust_domain_allocation_view(view: &PrimaryTrustDomainAllocationView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_trust_domain_allocation("code-21", "name-799");
        let view = to_primary_trust_domain_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_trust_domain_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
