use super::*;

use acme_kit::text;

/// How screens and exports show a capped delegation draft.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDelegationDraftView {
    pub code: String,
    pub label: String,
    pub category: String,
    pub updated_at: String,
    pub duration_minutes: String,
    pub title: String,
    pub country_text: String,
    pub created_at_text: String,
    pub status_label: String,
}

pub fn to_capped_delegation_draft_view(item: &CappedDelegationDraft) -> CappedDelegationDraftView {
    CappedDelegationDraftView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        category: text::truncate(&item.category, 40),
        updated_at: item.updated_at.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        title: text::truncate(&item.title, 120),
        country_text: text::truncate(&item.country, 80),
        created_at_text: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_delegation_draft_views(items: &[CappedDelegationDraft]) -> Vec<CappedDelegationDraftView> {
    let mut views: Vec<CappedDelegationDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_delegation_draft_view(item));
    }
    views
}

pub fn sum_capped_delegation_draft_view(view: &CappedDelegationDraftView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_duration_minutes as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_delegation_draft("code-421", "label-620");
        let view = to_capped_delegation_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_delegation_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
