use super::*;

use acme_kit::text;

/// How screens and exports show a global trust domain quota.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTrustDomainQuotaView {
    pub code: String,
    pub name: String,
    pub retries: String,
    pub pinned: String,
    pub quota_units: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_global_trust_domain_quota_view(item: &GlobalTrustDomainQuota) -> GlobalTrustDomainQuotaView {
    GlobalTrustDomainQuotaView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        retries: item.retries.to_string(),
        pinned: format!("{}", item.pinned),
        quota_units: item.quota_units.to_string(),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_trust_domain_quota_views(items: &[GlobalTrustDomainQuota]) -> Vec<GlobalTrustDomainQuotaView> {
    let mut views: Vec<GlobalTrustDomainQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_trust_domain_quota_view(item));
    }
    views
}

pub fn sum_global_trust_domain_quota_view(view: &GlobalTrustDomainQuotaView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_trust_domain_quota("code-799", "name-880");
        let view = to_global_trust_domain_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_trust_domain_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
