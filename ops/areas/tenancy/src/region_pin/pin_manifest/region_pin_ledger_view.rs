use super::*;

use acme_kit::text;

/// How screens and exports show a region pin ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinLedgerView {
    pub code_text: String,
    pub label: String,
    pub weight_text: String,
    pub phone_text: String,
    pub grace_days: String,
    pub archived: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_region_pin_ledger_view(item: &RegionPinLedger) -> RegionPinLedgerView {
    RegionPinLedgerView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        weight_text: format!("{:.2}", item.weight),
        phone_text: item.phone.clone(),
        grace_days: item.grace_days.to_string(),
        archived: format!("{}", item.archived),
        currency: text::truncate(&item.currency, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_region_pin_ledger_views(items: &[RegionPinLedger]) -> Vec<RegionPinLedgerView> {
    let mut views: Vec<RegionPinLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_region_pin_ledger_view(item));
    }
    views
}

pub fn sum_region_pin_ledger_view(view: &RegionPinLedgerView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_weight + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_region_pin_ledger("code-19", "label-43");
        let view = to_region_pin_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_region_pin_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
