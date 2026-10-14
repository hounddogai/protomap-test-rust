use super::*;

use acme_kit::text;

/// How screens and exports show a batched redaction draft.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionDraftView {
    pub code_text: String,
    pub label: String,
    pub maximum_text: String,
    pub unit: String,
    pub contact_email: String,
    pub starts_at: String,
    pub priority: String,
    pub burst: String,
    pub retries_text: String,
    pub status_label: String,
}

pub fn to_batched_redaction_draft_view(item: &BatchedRedactionDraft) -> BatchedRedactionDraftView {
    BatchedRedactionDraftView {
        code_text: text::truncate(&item.code, 120),
        label: item.label.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        unit: item.unit.clone(),
        contact_email: text::truncate(&item.contact_email, 64),
        starts_at: item.starts_at.to_string(),
        priority: item.priority.to_string(),
        burst: item.burst.to_string(),
        retries_text: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_redaction_draft_views(items: &[BatchedRedactionDraft]) -> Vec<BatchedRedactionDraftView> {
    let mut views: Vec<BatchedRedactionDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_redaction_draft_view(item));
    }
    views
}

pub fn sum_batched_redaction_draft_view(view: &BatchedRedactionDraftView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_maximum + (parsed_starts_at as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_redaction_draft("code-425", "label-784");
        let view = to_batched_redaction_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_redaction_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
