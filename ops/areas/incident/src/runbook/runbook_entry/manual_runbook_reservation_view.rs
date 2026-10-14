use super::*;

use acme_kit::text;

/// How screens and exports show a manual runbook reservation.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookReservationView {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub external_ref: String,
    pub namespace: String,
    pub sequence: String,
    pub budget_text: String,
    pub retries: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_manual_runbook_reservation_view(item: &ManualRunbookReservation) -> ManualRunbookReservationView {
    ManualRunbookReservationView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        locale: item.locale.clone(),
        external_ref: item.external_ref.clone(),
        namespace: item.namespace.clone(),
        sequence: item.sequence.to_string(),
        budget_text: format!("{:.2}", item.budget),
        retries: item.retries.to_string(),
        source: text::truncate(&item.source, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_runbook_reservation_views(items: &[ManualRunbookReservation]) -> Vec<ManualRunbookReservationView> {
    let mut views: Vec<ManualRunbookReservationView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_runbook_reservation_view(item));
    }
    views
}

pub fn sum_manual_runbook_reservation_view(view: &ManualRunbookReservationView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_budget + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_runbook_reservation("code-100", "label-43");
        let view = to_manual_runbook_reservation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_runbook_reservation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
