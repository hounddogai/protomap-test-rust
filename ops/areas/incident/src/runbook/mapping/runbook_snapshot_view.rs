use super::*;

use acme_kit::text;

/// How screens and exports show a runbook snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSnapshotView {
    pub code_text: String,
    pub title_text: String,
    pub attempts_text: String,
    pub updated_at: String,
    pub budget_text: String,
    pub margin_text: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_runbook_snapshot_view(item: &RunbookSnapshot) -> RunbookSnapshotView {
    RunbookSnapshotView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        attempts_text: item.attempts.to_string(),
        updated_at: item.updated_at.to_string(),
        budget_text: format!("{:.2}", item.budget),
        margin_text: format!("{:.2}", item.margin),
        locale: text::truncate(&item.locale, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_snapshot_views(items: &[RunbookSnapshot]) -> Vec<RunbookSnapshotView> {
    let mut views: Vec<RunbookSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_snapshot_view(item));
    }
    views
}

pub fn sum_runbook_snapshot_view(view: &RunbookSnapshotView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_updated_at as f64) + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_snapshot("code-787", "title-709");
        let view = to_runbook_snapshot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_snapshot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
