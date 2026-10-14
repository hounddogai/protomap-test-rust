use super::*;

use acme_kit::text;

/// How screens and exports show a fallback credential ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackCredentialLedgerView {
    pub code: String,
    pub name: String,
    pub retries: String,
    pub margin: String,
    pub expires_at: String,
    pub quota_units: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_fallback_credential_ledger_view(item: &FallbackCredentialLedger) -> FallbackCredentialLedgerView {
    FallbackCredentialLedgerView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        retries: item.retries.to_string(),
        margin: format!("{:.2}", item.margin),
        expires_at: item.expires_at.to_string(),
        quota_units: item.quota_units.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_credential_ledger_views(items: &[FallbackCredentialLedger]) -> Vec<FallbackCredentialLedgerView> {
    let mut views: Vec<FallbackCredentialLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_credential_ledger_view(item));
    }
    views
}

pub fn sum_fallback_credential_ledger_view(view: &FallbackCredentialLedgerView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_retries as f64 + parsed_margin + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_credential_ledger("code-87", "name-727");
        let view = to_fallback_credential_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_credential_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
