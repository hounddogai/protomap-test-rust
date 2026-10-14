use super::*;

/// How screens and exports show a pending region pin ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinLedgerView {
    pub code: String,
    pub name: String,
    pub host_text: String,
    pub maximum: String,
    pub minimum: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_pending_region_pin_ledger_view(item: &PendingRegionPinLedger) -> PendingRegionPinLedgerView {
    PendingRegionPinLedgerView {
        code: item.code.clone(),
        name: item.name.clone(),
        host_text: item.host.clone(),
        maximum: format!("{:.2}", item.maximum),
        minimum: format!("{:.2}", item.minimum),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_region_pin_ledger_views(items: &[PendingRegionPinLedger]) -> Vec<PendingRegionPinLedgerView> {
    let mut views: Vec<PendingRegionPinLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_region_pin_ledger_view(item));
    }
    views
}

pub fn sum_pending_region_pin_ledger_view(view: &PendingRegionPinLedgerView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_maximum + parsed_minimum + parsed_margin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_region_pin_ledger("code-905", "name-83");
        let view = to_pending_region_pin_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_region_pin_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
