use super::*;

use acme_kit::text;

/// How screens and exports show a freeze draft.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeDraftView {
    pub code_text: String,
    pub name: String,
    pub country: String,
    pub key_id_text: String,
    pub attempts: String,
    pub contact_email: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_freeze_draft_view(item: &FreezeDraft) -> FreezeDraftView {
    FreezeDraftView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        country: item.country.clone(),
        key_id_text: item.key_id.clone(),
        attempts: item.attempts.to_string(),
        contact_email: text::truncate(&item.contact_email, 120),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_draft_views(items: &[FreezeDraft]) -> Vec<FreezeDraftView> {
    let mut views: Vec<FreezeDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_draft_view(item));
    }
    views
}

pub fn sum_freeze_draft_view(view: &FreezeDraftView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_freeze_draft("code-665", "name-801");
        let view = to_freeze_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_freeze_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
