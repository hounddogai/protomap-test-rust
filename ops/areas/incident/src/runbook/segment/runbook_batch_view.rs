use super::*;

use acme_kit::text;

/// How screens and exports show a runbook batch.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookBatchView {
    pub code: String,
    pub title: String,
    pub expires_at: String,
    pub floor_value: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_runbook_batch_view(item: &RunbookBatch) -> RunbookBatchView {
    RunbookBatchView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 40),
        expires_at: item.expires_at.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_batch_views(items: &[RunbookBatch]) -> Vec<RunbookBatchView> {
    let mut views: Vec<RunbookBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_batch_view(item));
    }
    views
}

pub fn sum_runbook_batch_view(view: &RunbookBatchView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_batch("code-112", "title-126");
        let view = to_runbook_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
