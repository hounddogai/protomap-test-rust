use super::*;

use acme_kit::text;

/// How screens and exports show a global peering ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringLedgerView {
    pub code: String,
    pub name_text: String,
    pub key_id_text: String,
    pub label: String,
    pub currency: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_global_peering_ledger_view(item: &GlobalPeeringLedger) -> GlobalPeeringLedgerView {
    GlobalPeeringLedgerView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        key_id_text: text::truncate(&item.key_id, 80),
        label: text::truncate(&item.label, 64),
        currency: item.currency.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_peering_ledger_views(items: &[GlobalPeeringLedger]) -> Vec<GlobalPeeringLedgerView> {
    let mut views: Vec<GlobalPeeringLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_peering_ledger_view(item));
    }
    views
}

pub fn sum_global_peering_ledger_view(view: &GlobalPeeringLedgerView) -> Result<f64, String> {
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
        let item = new_global_peering_ledger("code-253", "name-507");
        let view = to_global_peering_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_peering_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
