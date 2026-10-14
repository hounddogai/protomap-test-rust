use super::*;

use acme_kit::text;

/// How screens and exports show a batched redaction summary.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedRedactionSummaryView {
    pub code: String,
    pub label: String,
    pub duration_minutes: String,
    pub key_id: String,
    pub approved: String,
    pub sequence: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_batched_redaction_summary_view(item: &BatchedRedactionSummary) -> BatchedRedactionSummaryView {
    BatchedRedactionSummaryView {
        code: item.code.clone(),
        label: item.label.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        key_id: item.key_id.clone(),
        approved: format!("{}", item.approved),
        sequence: item.sequence.to_string(),
        category_text: text::truncate(&item.category, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_redaction_summary_views(items: &[BatchedRedactionSummary]) -> Vec<BatchedRedactionSummaryView> {
    let mut views: Vec<BatchedRedactionSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_redaction_summary_view(item));
    }
    views
}

pub fn sum_batched_redaction_summary_view(view: &BatchedRedactionSummaryView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_redaction_summary("code-58", "label-111");
        let view = to_batched_redaction_summary_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_redaction_summary_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
