use super::*;

use acme_kit::text;

/// How screens and exports show a identity ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityLedgerView {
    pub code_text: String,
    pub name: String,
    pub archived: String,
    pub ends_at_text: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_identity_ledger_view(item: &IdentityLedger) -> IdentityLedgerView {
    IdentityLedgerView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 64),
        archived: format!("{}", item.archived),
        ends_at_text: item.ends_at.to_string(),
        fingerprint_text: item.fingerprint.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_ledger_views(items: &[IdentityLedger]) -> Vec<IdentityLedgerView> {
    let mut views: Vec<IdentityLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_ledger_view(item));
    }
    views
}

pub fn sum_identity_ledger_view(view: &IdentityLedgerView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_ends_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_ledger("code-340", "name-258");
        let view = to_identity_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
