use super::*;

use acme_kit::text;

/// How screens and exports show a lenient action item ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemLedgerView {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub port: String,
    pub threshold: String,
    pub scope_text: String,
    pub external_ref: String,
    pub amount_text: String,
    pub revision: String,
    pub issuer_text: String,
    pub region_text: String,
    pub status_label: String,
}

pub fn to_lenient_action_item_ledger_view(item: &LenientActionItemLedger) -> LenientActionItemLedgerView {
    LenientActionItemLedgerView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        cipher: item.cipher.clone(),
        port: item.port.to_string(),
        threshold: format!("{:.2}", item.threshold),
        scope_text: item.scope.clone(),
        external_ref: item.external_ref.clone(),
        amount_text: format!("{:.2}", item.amount),
        revision: item.revision.to_string(),
        issuer_text: item.issuer.clone(),
        region_text: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_action_item_ledger_views(items: &[LenientActionItemLedger]) -> Vec<LenientActionItemLedgerView> {
    let mut views: Vec<LenientActionItemLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_action_item_ledger_view(item));
    }
    views
}

pub fn sum_lenient_action_item_ledger_view(view: &LenientActionItemLedgerView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_port as f64 + parsed_threshold + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_lenient_action_item_ledger("code-595", "title-643");
        let view = to_lenient_action_item_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_lenient_action_item_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
