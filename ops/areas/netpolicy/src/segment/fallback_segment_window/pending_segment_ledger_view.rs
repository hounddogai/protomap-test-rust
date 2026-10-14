use super::*;

use acme_kit::text;

/// How screens and exports show a pending segment ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentLedgerView {
    pub code: String,
    pub name: String,
    pub reviewer_text: String,
    pub external_ref: String,
    pub archived: String,
    pub minimum: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_pending_segment_ledger_view(item: &PendingSegmentLedger) -> PendingSegmentLedgerView {
    PendingSegmentLedgerView {
        code: item.code.clone(),
        name: item.name.clone(),
        reviewer_text: item.reviewer.clone(),
        external_ref: text::truncate(&item.external_ref, 40),
        archived: format!("{}", item.archived),
        minimum: format!("{:.2}", item.minimum),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_segment_ledger_views(items: &[PendingSegmentLedger]) -> Vec<PendingSegmentLedgerView> {
    let mut views: Vec<PendingSegmentLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_segment_ledger_view(item));
    }
    views
}

pub fn sum_pending_segment_ledger_view(view: &PendingSegmentLedgerView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_minimum + (parsed_updated_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_segment_ledger("code-957", "name-864");
        let view = to_pending_segment_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_segment_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
