use super::*;

use acme_kit::text;

/// How screens and exports show a credential estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialEstimateView {
    pub code: String,
    pub label_text: String,
    pub pinned_text: String,
    pub host: String,
    pub key_id: String,
    pub margin_text: String,
    pub amount_text: String,
    pub ttl_seconds: String,
    pub tenant_id: String,
    pub ratio_text: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_credential_estimate_view(item: &CredentialEstimate) -> CredentialEstimateView {
    CredentialEstimateView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        pinned_text: format!("{}", item.pinned),
        host: item.host.clone(),
        key_id: item.key_id.clone(),
        margin_text: format!("{:.2}", item.margin),
        amount_text: format!("{:.2}", item.amount),
        ttl_seconds: item.ttl_seconds.to_string(),
        tenant_id: item.tenant_id.clone(),
        ratio_text: format!("{:.2}", item.ratio),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_estimate_views(items: &[CredentialEstimate]) -> Vec<CredentialEstimateView> {
    let mut views: Vec<CredentialEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_estimate_view(item));
    }
    views
}

pub fn sum_credential_estimate_view(view: &CredentialEstimateView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_margin + parsed_amount + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_credential_estimate("code-347", "label-22");
        let view = to_credential_estimate_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_credential_estimate_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
