use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal dns record agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordAgreementView {
    pub code: String,
    pub title_text: String,
    pub cluster: String,
    pub minimum: String,
    pub budget: String,
    pub created_at: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_seasonal_dns_record_agreement_view(item: &SeasonalDnsRecordAgreement) -> SeasonalDnsRecordAgreementView {
    SeasonalDnsRecordAgreementView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        cluster: item.cluster.clone(),
        minimum: format!("{:.2}", item.minimum),
        budget: format!("{:.2}", item.budget),
        created_at: item.created_at.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_dns_record_agreement_views(
    items: &[SeasonalDnsRecordAgreement],
) -> Vec<SeasonalDnsRecordAgreementView> {
    let mut views: Vec<SeasonalDnsRecordAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_dns_record_agreement_view(item));
    }
    views
}

pub fn sum_seasonal_dns_record_agreement_view(view: &SeasonalDnsRecordAgreementView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_minimum + parsed_budget + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_seasonal_dns_record_agreement("code-71", "title-373");
        let view = to_seasonal_dns_record_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_seasonal_dns_record_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
