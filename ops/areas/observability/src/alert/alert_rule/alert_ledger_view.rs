use super::*;

use acme_kit::text;

/// How screens and exports show a alert ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertLedgerView {
    pub code: String,
    pub label_text: String,
    pub budget: String,
    pub rotation_days: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_alert_ledger_view(item: &AlertLedger) -> AlertLedgerView {
    AlertLedgerView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 80),
        budget: format!("{:.2}", item.budget),
        rotation_days: item.rotation_days.to_string(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_ledger_views(items: &[AlertLedger]) -> Vec<AlertLedgerView> {
    let mut views: Vec<AlertLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_ledger_view(item));
    }
    views
}

pub fn sum_alert_ledger_view(view: &AlertLedgerView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_budget + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_alert_ledger("code-997", "label-261");
        let view = to_alert_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_alert_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
