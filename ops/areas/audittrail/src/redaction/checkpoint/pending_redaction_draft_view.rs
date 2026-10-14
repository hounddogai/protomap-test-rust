use super::*;

/// How screens and exports show a pending redaction draft.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRedactionDraftView {
    pub code_text: String,
    pub label: String,
    pub duration_minutes_text: String,
    pub currency: String,
    pub active: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_pending_redaction_draft_view(item: &PendingRedactionDraft) -> PendingRedactionDraftView {
    PendingRedactionDraftView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        duration_minutes_text: item.duration_minutes.to_string(),
        currency: item.currency.clone(),
        active: format!("{}", item.active),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_redaction_draft_views(items: &[PendingRedactionDraft]) -> Vec<PendingRedactionDraftView> {
    let mut views: Vec<PendingRedactionDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_redaction_draft_view(item));
    }
    views
}

pub fn sum_pending_redaction_draft_view(view: &PendingRedactionDraftView) -> Result<f64, String> {
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_duration_minutes as f64 + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_redaction_draft("code-472", "label-736");
        let view = to_pending_redaction_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_redaction_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
