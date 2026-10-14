use super::*;

use acme_kit::text;

/// How screens and exports show a global trust domain tier.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTrustDomainTierView {
    pub code: String,
    pub name: String,
    pub capacity: String,
    pub contact_email: String,
    pub starts_at_text: String,
    pub checksum: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_global_trust_domain_tier_view(item: &GlobalTrustDomainTier) -> GlobalTrustDomainTierView {
    GlobalTrustDomainTierView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 64),
        capacity: item.capacity.to_string(),
        contact_email: item.contact_email.clone(),
        starts_at_text: item.starts_at.to_string(),
        checksum: text::truncate(&item.checksum, 120),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_trust_domain_tier_views(items: &[GlobalTrustDomainTier]) -> Vec<GlobalTrustDomainTierView> {
    let mut views: Vec<GlobalTrustDomainTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_trust_domain_tier_view(item));
    }
    views
}

pub fn sum_global_trust_domain_tier_view(view: &GlobalTrustDomainTierView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_starts_at as f64) + parsed_minimum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_trust_domain_tier("code-379", "name-32");
        let view = to_global_trust_domain_tier_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_trust_domain_tier_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
