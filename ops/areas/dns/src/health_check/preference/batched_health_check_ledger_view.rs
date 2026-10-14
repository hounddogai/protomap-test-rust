use super::*;

use acme_kit::text;

/// How screens and exports show a batched health check ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckLedgerView {
    pub code: String,
    pub name_text: String,
    pub archived_text: String,
    pub margin: String,
    pub scope: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_batched_health_check_ledger_view(item: &BatchedHealthCheckLedger) -> BatchedHealthCheckLedgerView {
    BatchedHealthCheckLedgerView {
        code: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        archived_text: format!("{}", item.archived),
        margin: format!("{:.2}", item.margin),
        scope: item.scope.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_health_check_ledger_views(items: &[BatchedHealthCheckLedger]) -> Vec<BatchedHealthCheckLedgerView> {
    let mut views: Vec<BatchedHealthCheckLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_health_check_ledger_view(item));
    }
    views
}

pub fn sum_batched_health_check_ledger_view(view: &BatchedHealthCheckLedgerView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_margin + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_health_check_ledger("code-291", "name-469");
        let view = to_batched_health_check_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_health_check_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
