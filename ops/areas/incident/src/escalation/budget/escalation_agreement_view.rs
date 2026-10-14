use super::*;

use acme_kit::text;

/// How screens and exports show a escalation agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAgreementView {
    pub code_text: String,
    pub name: String,
    pub burst_text: String,
    pub maximum: String,
    pub expires_at: String,
    pub quantity: String,
    pub label_text: String,
    pub status_label: String,
}

pub fn to_escalation_agreement_view(item: &EscalationAgreement) -> EscalationAgreementView {
    EscalationAgreementView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        burst_text: item.burst.to_string(),
        maximum: format!("{:.2}", item.maximum),
        expires_at: item.expires_at.to_string(),
        quantity: item.quantity.to_string(),
        label_text: item.label.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_escalation_agreement_views(items: &[EscalationAgreement]) -> Vec<EscalationAgreementView> {
    let mut views: Vec<EscalationAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_escalation_agreement_view(item));
    }
    views
}

pub fn sum_escalation_agreement_view(view: &EscalationAgreementView) -> Result<f64, String> {
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_burst as f64 + parsed_maximum + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_escalation_agreement("code-935", "name-983");
        let view = to_escalation_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_escalation_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
