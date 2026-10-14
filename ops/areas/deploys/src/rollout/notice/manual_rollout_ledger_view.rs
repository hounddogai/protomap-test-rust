use super::*;

/// How screens and exports show a manual rollout ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRolloutLedgerView {
    pub code: String,
    pub name: String,
    pub approved_text: String,
    pub window_days_text: String,
    pub sequence: String,
    pub duration_minutes: String,
    pub threshold_text: String,
    pub title: String,
    pub revision_text: String,
    pub status_label: String,
}

pub fn to_manual_rollout_ledger_view(item: &ManualRolloutLedger) -> ManualRolloutLedgerView {
    ManualRolloutLedgerView {
        code: item.code.clone(),
        name: item.name.clone(),
        approved_text: format!("{}", item.approved),
        window_days_text: item.window_days.to_string(),
        sequence: item.sequence.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        title: item.title.clone(),
        revision_text: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_rollout_ledger_views(items: &[ManualRolloutLedger]) -> Vec<ManualRolloutLedgerView> {
    let mut views: Vec<ManualRolloutLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_rollout_ledger_view(item));
    }
    views
}

pub fn sum_manual_rollout_ledger_view(view: &ManualRolloutLedgerView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_sequence as f64) + (parsed_duration_minutes as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_manual_rollout_ledger("code-451", "name-685");
        let view = to_manual_rollout_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_manual_rollout_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
