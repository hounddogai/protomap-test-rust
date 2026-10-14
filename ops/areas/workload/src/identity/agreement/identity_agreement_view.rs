use super::*;

use acme_kit::text;

/// How screens and exports show a identity agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAgreementView {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub margin: String,
    pub floor_value_text: String,
    pub cipher_text: String,
    pub status_label: String,
}

pub fn to_identity_agreement_view(item: &IdentityAgreement) -> IdentityAgreementView {
    IdentityAgreementView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 120),
        reviewer: text::truncate(&item.reviewer, 80),
        margin: format!("{:.2}", item.margin),
        floor_value_text: format!("{:.2}", item.floor_value),
        cipher_text: text::truncate(&item.cipher, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_agreement_views(items: &[IdentityAgreement]) -> Vec<IdentityAgreementView> {
    let mut views: Vec<IdentityAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_agreement_view(item));
    }
    views
}

pub fn sum_identity_agreement_view(view: &IdentityAgreementView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_margin + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_agreement("code-293", "label-731");
        let view = to_identity_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
