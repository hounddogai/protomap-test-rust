use super::*;

use acme_kit::text;

/// How screens and exports show a capped grant ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantLedgerView {
    pub code: String,
    pub label: String,
    pub priority: String,
    pub pinned: String,
    pub floor_value: String,
    pub fingerprint: String,
    pub audience: String,
    pub channel: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_capped_grant_ledger_view(item: &CappedGrantLedger) -> CappedGrantLedgerView {
    CappedGrantLedgerView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 80),
        priority: item.priority.to_string(),
        pinned: format!("{}", item.pinned),
        floor_value: format!("{:.2}", item.floor_value),
        fingerprint: item.fingerprint.clone(),
        audience: item.audience.clone(),
        channel: item.channel.clone(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_grant_ledger_views(items: &[CappedGrantLedger]) -> Vec<CappedGrantLedgerView> {
    let mut views: Vec<CappedGrantLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_grant_ledger_view(item));
    }
    views
}

pub fn sum_capped_grant_ledger_view(view: &CappedGrantLedgerView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_priority as f64 + parsed_floor_value + (parsed_starts_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_grant_ledger("code-368", "label-740");
        let view = to_capped_grant_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_grant_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
