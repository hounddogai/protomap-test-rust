use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain slot.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSlotView {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub checksum: String,
    pub key_id: String,
    pub host_text: String,
    pub description: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_trust_domain_slot_view(item: &TrustDomainSlot) -> TrustDomainSlotView {
    TrustDomainSlotView {
        code: item.code.clone(),
        label: item.label.clone(),
        tenant_id: item.tenant_id.clone(),
        checksum: item.checksum.clone(),
        key_id: item.key_id.clone(),
        host_text: text::truncate(&item.host, 80),
        description: item.description.clone(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_slot_views(items: &[TrustDomainSlot]) -> Vec<TrustDomainSlotView> {
    let mut views: Vec<TrustDomainSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_slot_view(item));
    }
    views
}

pub fn sum_trust_domain_slot_view(view: &TrustDomainSlotView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_slot("code-80", "label-735");
        let view = to_trust_domain_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
