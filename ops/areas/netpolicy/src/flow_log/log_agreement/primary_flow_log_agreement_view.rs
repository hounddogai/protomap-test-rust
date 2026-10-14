use super::*;

use acme_kit::text;

/// How screens and exports show a primary flow log agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFlowLogAgreementView {
    pub code_text: String,
    pub name_text: String,
    pub phone: String,
    pub key_id_text: String,
    pub source: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_primary_flow_log_agreement_view(item: &PrimaryFlowLogAgreement) -> PrimaryFlowLogAgreementView {
    PrimaryFlowLogAgreementView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        phone: text::truncate(&item.phone, 120),
        key_id_text: item.key_id.clone(),
        source: item.source.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_flow_log_agreement_views(items: &[PrimaryFlowLogAgreement]) -> Vec<PrimaryFlowLogAgreementView> {
    let mut views: Vec<PrimaryFlowLogAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_flow_log_agreement_view(item));
    }
    views
}

pub fn sum_primary_flow_log_agreement_view(view: &PrimaryFlowLogAgreementView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_flow_log_agreement("code-95", "name-9");
        let view = to_primary_flow_log_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_flow_log_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
