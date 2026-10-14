use super::*;

use acme_kit::text;

/// How screens and exports show a manual dns record agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordAgreementView {
    pub code: String,
    pub title: String,
    pub grace_days_text: String,
    pub maximum_text: String,
    pub rate: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_manual_dns_record_agreement_view(item: &ManualDnsRecordAgreement) -> ManualDnsRecordAgreementView {
    ManualDnsRecordAgreementView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        grace_days_text: item.grace_days.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        rate: format!("{:.2}", item.rate),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_dns_record_agreement_views(items: &[ManualDnsRecordAgreement]) -> Vec<ManualDnsRecordAgreementView> {
    let mut views: Vec<ManualDnsRecordAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_dns_record_agreement_view(item));
    }
    views
}

pub fn sum_manual_dns_record_agreement_view(view: &ManualDnsRecordAgreementView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_grace_days as f64 + parsed_maximum + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_dns_record_agreement("code-572", "title-764");
        let view = to_manual_dns_record_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_dns_record_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
