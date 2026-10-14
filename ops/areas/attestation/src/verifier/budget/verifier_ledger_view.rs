use super::*;

use acme_kit::text;

/// How screens and exports show a verifier ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierLedgerView {
    pub code: String,
    pub label_text: String,
    pub key_id_text: String,
    pub quantity: String,
    pub active: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_verifier_ledger_view(item: &VerifierLedger) -> VerifierLedgerView {
    VerifierLedgerView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        key_id_text: item.key_id.clone(),
        quantity: item.quantity.to_string(),
        active: format!("{}", item.active),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_ledger_views(items: &[VerifierLedger]) -> Vec<VerifierLedgerView> {
    let mut views: Vec<VerifierLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_ledger_view(item));
    }
    views
}

pub fn sum_verifier_ledger_view(view: &VerifierLedgerView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_ledger("code-975", "label-717");
        let view = to_verifier_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
