use super::*;

use acme_kit::text;

/// How screens and exports show a failover ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverLedgerView {
    pub code_text: String,
    pub title: String,
    pub host_text: String,
    pub grace_days_text: String,
    pub external_ref: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_failover_ledger_view(item: &FailoverLedger) -> FailoverLedgerView {
    FailoverLedgerView {
        code_text: text::truncate(&item.code, 80),
        title: item.title.clone(),
        host_text: item.host.clone(),
        grace_days_text: item.grace_days.to_string(),
        external_ref: text::truncate(&item.external_ref, 120),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_ledger_views(items: &[FailoverLedger]) -> Vec<FailoverLedgerView> {
    let mut views: Vec<FailoverLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_ledger_view(item));
    }
    views
}

pub fn sum_failover_ledger_view(view: &FailoverLedgerView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_failover_ledger("code-775", "title-394");
        let view = to_failover_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_failover_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
