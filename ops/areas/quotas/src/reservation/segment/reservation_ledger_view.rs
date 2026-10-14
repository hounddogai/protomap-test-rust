use super::*;

use acme_kit::text;

/// How screens and exports show a reservation ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationLedgerView {
    pub code: String,
    pub name_text: String,
    pub revision: String,
    pub owner_id_text: String,
    pub version: String,
    pub active: String,
    pub burst_text: String,
    pub region: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_reservation_ledger_view(item: &ReservationLedger) -> ReservationLedgerView {
    ReservationLedgerView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        revision: item.revision.to_string(),
        owner_id_text: text::truncate(&item.owner_id, 40),
        version: item.version.to_string(),
        active: format!("{}", item.active),
        burst_text: item.burst.to_string(),
        region: item.region.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_reservation_ledger_views(items: &[ReservationLedger]) -> Vec<ReservationLedgerView> {
    let mut views: Vec<ReservationLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_reservation_ledger_view(item));
    }
    views
}

pub fn sum_reservation_ledger_view(view: &ReservationLedgerView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_version as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_reservation_ledger("code-423", "name-829");
        let view = to_reservation_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_reservation_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
