use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled isolation ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIsolationLedgerView {
    pub code: String,
    pub name_text: String,
    pub created_at: String,
    pub description: String,
    pub quantity_text: String,
    pub timezone: String,
    pub priority: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_scheduled_isolation_ledger_view(item: &ScheduledIsolationLedger) -> ScheduledIsolationLedgerView {
    ScheduledIsolationLedgerView {
        code: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        created_at: item.created_at.to_string(),
        description: item.description.clone(),
        quantity_text: item.quantity.to_string(),
        timezone: item.timezone.clone(),
        priority: item.priority.to_string(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_isolation_ledger_views(items: &[ScheduledIsolationLedger]) -> Vec<ScheduledIsolationLedgerView> {
    let mut views: Vec<ScheduledIsolationLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_isolation_ledger_view(item));
    }
    views
}

pub fn sum_scheduled_isolation_ledger_view(view: &ScheduledIsolationLedgerView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_quantity as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_scheduled_isolation_ledger("code-825", "name-819");
        let view = to_scheduled_isolation_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_scheduled_isolation_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
