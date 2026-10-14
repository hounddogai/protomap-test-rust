use super::*;

use acme_kit::text;

/// How screens and exports show a credential agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAgreementView {
    pub code: String,
    pub label: String,
    pub maximum: String,
    pub grace_days: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_credential_agreement_view(item: &CredentialAgreement) -> CredentialAgreementView {
    CredentialAgreementView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        maximum: format!("{:.2}", item.maximum),
        grace_days: item.grace_days.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_agreement_views(items: &[CredentialAgreement]) -> Vec<CredentialAgreementView> {
    let mut views: Vec<CredentialAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_agreement_view(item));
    }
    views
}

pub fn sum_credential_agreement_view(view: &CredentialAgreementView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_maximum + (parsed_grace_days as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_agreement("code-277", "label-966");
        let view = to_credential_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
