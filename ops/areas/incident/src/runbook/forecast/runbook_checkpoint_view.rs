use super::*;

use acme_kit::text;

/// How screens and exports show a runbook checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookCheckpointView {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub external_ref_text: String,
    pub cluster: String,
    pub currency: String,
    pub country: String,
    pub approved: String,
    pub weight_text: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_runbook_checkpoint_view(item: &RunbookCheckpoint) -> RunbookCheckpointView {
    RunbookCheckpointView {
        code: item.code.clone(),
        label: item.label.clone(),
        key_id: text::truncate(&item.key_id, 64),
        external_ref_text: text::truncate(&item.external_ref, 80),
        cluster: text::truncate(&item.cluster, 64),
        currency: text::truncate(&item.currency, 80),
        country: item.country.clone(),
        approved: format!("{}", item.approved),
        weight_text: format!("{:.2}", item.weight),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_checkpoint_views(items: &[RunbookCheckpoint]) -> Vec<RunbookCheckpointView> {
    let mut views: Vec<RunbookCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_checkpoint_view(item));
    }
    views
}

pub fn sum_runbook_checkpoint_view(view: &RunbookCheckpointView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_weight + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_checkpoint("code-689", "label-236");
        let view = to_runbook_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
