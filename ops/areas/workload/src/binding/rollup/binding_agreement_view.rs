use super::*;

use acme_kit::text;

/// How screens and exports show a binding agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingAgreementView {
    pub code: String,
    pub name_text: String,
    pub grace_days_text: String,
    pub priority: String,
    pub external_ref: String,
    pub namespace: String,
    pub limit: String,
    pub retries: String,
    pub checksum: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_binding_agreement_view(item: &BindingAgreement) -> BindingAgreementView {
    BindingAgreementView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        grace_days_text: item.grace_days.to_string(),
        priority: item.priority.to_string(),
        external_ref: item.external_ref.clone(),
        namespace: item.namespace.clone(),
        limit: item.limit.to_string(),
        retries: item.retries.to_string(),
        checksum: item.checksum.clone(),
        channel: text::truncate(&item.channel, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_agreement_views(items: &[BindingAgreement]) -> Vec<BindingAgreementView> {
    let mut views: Vec<BindingAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_agreement_view(item));
    }
    views
}

pub fn sum_binding_agreement_view(view: &BindingAgreementView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_priority as f64) + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_binding_agreement("code-98", "name-865");
        let view = to_binding_agreement_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_binding_agreement_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
