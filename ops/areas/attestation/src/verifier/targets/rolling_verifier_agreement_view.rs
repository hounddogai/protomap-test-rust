use super::*;

use acme_kit::text;

/// How screens and exports show a rolling verifier agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierAgreementView {
    pub code: String,
    pub title: String,
    pub name: String,
    pub ttl_seconds: String,
    pub revision: String,
    pub locale: String,
    pub checksum: String,
    pub amount: String,
    pub pinned: String,
    pub approved: String,
    pub rate_text: String,
    pub status_label: String,
}

pub fn to_rolling_verifier_agreement_view(item: &RollingVerifierAgreement) -> RollingVerifierAgreementView {
    RollingVerifierAgreementView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        name: text::truncate(&item.name, 80),
        ttl_seconds: item.ttl_seconds.to_string(),
        revision: item.revision.to_string(),
        locale: text::truncate(&item.locale, 120),
        checksum: item.checksum.clone(),
        amount: format!("{:.2}", item.amount),
        pinned: format!("{}", item.pinned),
        approved: format!("{}", item.approved),
        rate_text: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_verifier_agreement_views(items: &[RollingVerifierAgreement]) -> Vec<RollingVerifierAgreementView> {
    let mut views: Vec<RollingVerifierAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_verifier_agreement_view(item));
    }
    views
}

pub fn sum_rolling_verifier_agreement_view(view: &RollingVerifierAgreementView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_revision as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_verifier_agreement("code-152", "title-966");
        let view = to_rolling_verifier_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_verifier_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
