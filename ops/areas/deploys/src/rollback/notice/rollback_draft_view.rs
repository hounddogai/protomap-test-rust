use super::*;

use acme_kit::text;

/// How screens and exports show a rollback draft.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDraftView {
    pub code: String,
    pub name: String,
    pub updated_at_text: String,
    pub currency: String,
    pub cluster: String,
    pub attempts: String,
    pub region: String,
    pub timezone_text: String,
    pub status_label: String,
}

pub fn to_rollback_draft_view(item: &RollbackDraft) -> RollbackDraftView {
    RollbackDraftView {
        code: item.code.clone(),
        name: item.name.clone(),
        updated_at_text: item.updated_at.to_string(),
        currency: item.currency.clone(),
        cluster: text::truncate(&item.cluster, 64),
        attempts: item.attempts.to_string(),
        region: item.region.clone(),
        timezone_text: item.timezone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_draft_views(items: &[RollbackDraft]) -> Vec<RollbackDraftView> {
    let mut views: Vec<RollbackDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_draft_view(item));
    }
    views
}

pub fn sum_rollback_draft_view(view: &RollbackDraftView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollback_draft("code-992", "name-744");
        let view = to_rollback_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollback_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
