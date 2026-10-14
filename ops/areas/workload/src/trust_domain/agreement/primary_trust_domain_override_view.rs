use super::*;

use acme_kit::text;

/// How screens and exports show a primary trust domain override.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainOverrideView {
    pub code_text: String,
    pub label_text: String,
    pub attempts_text: String,
    pub audience_text: String,
    pub tenant_id_text: String,
    pub owner_id: String,
    pub port: String,
    pub quantity: String,
    pub region: String,
    pub starts_at: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_primary_trust_domain_override_view(item: &PrimaryTrustDomainOverride) -> PrimaryTrustDomainOverrideView {
    PrimaryTrustDomainOverrideView {
        code_text: text::truncate(&item.code, 80),
        label_text: text::truncate(&item.label, 80),
        attempts_text: item.attempts.to_string(),
        audience_text: item.audience.clone(),
        tenant_id_text: item.tenant_id.clone(),
        owner_id: text::truncate(&item.owner_id, 80),
        port: item.port.to_string(),
        quantity: item.quantity.to_string(),
        region: item.region.clone(),
        starts_at: item.starts_at.to_string(),
        namespace: text::truncate(&item.namespace, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_trust_domain_override_views(
    items: &[PrimaryTrustDomainOverride],
) -> Vec<PrimaryTrustDomainOverrideView> {
    let mut views: Vec<PrimaryTrustDomainOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_trust_domain_override_view(item));
    }
    views
}

pub fn sum_primary_trust_domain_override_view(view: &PrimaryTrustDomainOverrideView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_port as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_trust_domain_override("code-34", "label-861");
        let view = to_primary_trust_domain_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_trust_domain_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
