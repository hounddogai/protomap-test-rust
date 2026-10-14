use super::*;

use acme_kit::text;

/// How screens and exports show a tiered sink ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSinkLedgerView {
    pub code_text: String,
    pub title: String,
    pub region: String,
    pub active: String,
    pub fingerprint: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_tiered_sink_ledger_view(item: &TieredSinkLedger) -> TieredSinkLedgerView {
    TieredSinkLedgerView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        region: text::truncate(&item.region, 40),
        active: format!("{}", item.active),
        fingerprint: text::truncate(&item.fingerprint, 64),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_sink_ledger_views(items: &[TieredSinkLedger]) -> Vec<TieredSinkLedgerView> {
    let mut views: Vec<TieredSinkLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_sink_ledger_view(item));
    }
    views
}

pub fn sum_tiered_sink_ledger_view(view: &TieredSinkLedgerView) -> Result<f64, String> {
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
        let item = new_tiered_sink_ledger("code-588", "title-190");
        let view = to_tiered_sink_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_sink_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
