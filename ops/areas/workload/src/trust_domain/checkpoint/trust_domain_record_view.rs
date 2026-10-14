use super::*;

use acme_kit::text;

/// How screens and exports show a trust domain record.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRecordView {
    pub code: String,
    pub label: String,
    pub capacity_text: String,
    pub active: String,
    pub quota_units: String,
    pub currency_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_trust_domain_record_view(item: &TrustDomainRecord) -> TrustDomainRecordView {
    TrustDomainRecordView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        capacity_text: item.capacity.to_string(),
        active: format!("{}", item.active),
        quota_units: item.quota_units.to_string(),
        currency_text: item.currency.clone(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_trust_domain_record_views(items: &[TrustDomainRecord]) -> Vec<TrustDomainRecordView> {
    let mut views: Vec<TrustDomainRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_trust_domain_record_view(item));
    }
    views
}

pub fn sum_trust_domain_record_view(view: &TrustDomainRecordView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_trust_domain_record("code-487", "label-74");
        let view = to_trust_domain_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_trust_domain_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
