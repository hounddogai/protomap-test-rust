use super::*;

/// How screens and exports show a weighted trust domain agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainAgreementView {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub rotation_days_text: String,
    pub approved: String,
    pub sequence: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_weighted_trust_domain_agreement_view(
    item: &WeightedTrustDomainAgreement,
) -> WeightedTrustDomainAgreementView {
    WeightedTrustDomainAgreementView {
        code: item.code.clone(),
        label: item.label.clone(),
        issuer: item.issuer.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        approved: format!("{}", item.approved),
        sequence: item.sequence.to_string(),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_trust_domain_agreement_views(
    items: &[WeightedTrustDomainAgreement],
) -> Vec<WeightedTrustDomainAgreementView> {
    let mut views: Vec<WeightedTrustDomainAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_trust_domain_agreement_view(item));
    }
    views
}

pub fn sum_weighted_trust_domain_agreement_view(view: &WeightedTrustDomainAgreementView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_sequence as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_trust_domain_agreement("code-25", "label-494");
        let view = to_weighted_trust_domain_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_trust_domain_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
