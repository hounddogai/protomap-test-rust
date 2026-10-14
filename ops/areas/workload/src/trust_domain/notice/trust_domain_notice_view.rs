use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain notice.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainNoticeView {
    pub code: String,
    pub title_text: String,
    pub ttl_seconds: String,
    pub currency_text: String,
    pub cost: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_trust_domain_notice_view(item: &TrustDomainNotice) -> TrustDomainNoticeView {
    TrustDomainNoticeView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 80),
        ttl_seconds: item.ttl_seconds.to_string(),
        currency_text: text::truncate(&item.currency, 40),
        cost: format!("{:.2}", item.cost),
        checksum: text::truncate(&item.checksum, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_notice_views(items: &[TrustDomainNotice]) -> Vec<TrustDomainNoticeView> {
    let mut views: Vec<TrustDomainNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_notice_view(item));
    }
    views
}

pub fn sum_trust_domain_notice_view(view: &TrustDomainNoticeView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_notice("code-357", "title-27");
        let view = to_trust_domain_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
