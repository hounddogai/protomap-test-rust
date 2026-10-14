use super::*;

use acme_kit::text;

/// How screens and exports show a pending budget burn record.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingBudgetBurnRecordView {
    pub code: String,
    pub name_text: String,
    pub label: String,
    pub source: String,
    pub sequence: String,
    pub window_days: String,
    pub contact_email: String,
    pub burst_text: String,
    pub scope: String,
    pub grace_days: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_pending_budget_burn_record_view(item: &PendingBudgetBurnRecord) -> PendingBudgetBurnRecordView {
    PendingBudgetBurnRecordView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        label: item.label.clone(),
        source: item.source.clone(),
        sequence: item.sequence.to_string(),
        window_days: item.window_days.to_string(),
        contact_email: text::truncate(&item.contact_email, 64),
        burst_text: item.burst.to_string(),
        scope: text::truncate(&item.scope, 80),
        grace_days: item.grace_days.to_string(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_budget_burn_record_views(items: &[PendingBudgetBurnRecord]) -> Vec<PendingBudgetBurnRecordView> {
    let mut views: Vec<PendingBudgetBurnRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_budget_burn_record_view(item));
    }
    views
}

pub fn sum_pending_budget_burn_record_view(view: &PendingBudgetBurnRecordView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_burst = view
        .burst_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_window_days as f64) + (parsed_burst as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_budget_burn_record("code-949", "name-84");
        let view = to_pending_budget_burn_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_budget_burn_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
