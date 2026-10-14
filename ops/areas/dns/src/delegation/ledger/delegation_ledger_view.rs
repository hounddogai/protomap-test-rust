use super::*;

/// How screens and exports show a delegation ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationLedgerView {
    pub code: String,
    pub label: String,
    pub issuer_text: String,
    pub burst: String,
    pub batch_size: String,
    pub priority_text: String,
    pub status_label: String,
}

pub fn to_delegation_ledger_view(item: &DelegationLedger) -> DelegationLedgerView {
    DelegationLedgerView {
        code: item.code.clone(),
        label: item.label.clone(),
        issuer_text: item.issuer.clone(),
        burst: item.burst.to_string(),
        batch_size: item.batch_size.to_string(),
        priority_text: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_ledger_views(items: &[DelegationLedger]) -> Vec<DelegationLedgerView> {
    let mut views: Vec<DelegationLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_ledger_view(item));
    }
    views
}

pub fn sum_delegation_ledger_view(view: &DelegationLedgerView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_batch_size as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_ledger("code-247", "label-559");
        let view = to_delegation_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
