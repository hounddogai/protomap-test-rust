use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain quota.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainQuotaView {
    pub code: String,
    pub name: String,
    pub visible: String,
    pub cipher: String,
    pub channel: String,
    pub notes: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_trust_domain_quota_view(item: &TrustDomainQuota) -> TrustDomainQuotaView {
    TrustDomainQuotaView {
        code: item.code.clone(),
        name: item.name.clone(),
        visible: format!("{}", item.visible),
        cipher: text::truncate(&item.cipher, 40),
        channel: text::truncate(&item.channel, 80),
        notes: item.notes.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_quota_views(items: &[TrustDomainQuota]) -> Vec<TrustDomainQuotaView> {
    let mut views: Vec<TrustDomainQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_quota_view(item));
    }
    views
}

pub fn sum_trust_domain_quota_view(view: &TrustDomainQuotaView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_quota("code-362", "name-849");
        let view = to_trust_domain_quota_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_quota_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
