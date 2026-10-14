use super::*;

use acme_kit::text;

/// How screens and exports show a default action item draft.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemDraftView {
    pub code: String,
    pub label: String,
    pub source: String,
    pub priority: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_default_action_item_draft_view(item: &DefaultActionItemDraft) -> DefaultActionItemDraftView {
    DefaultActionItemDraftView {
        code: item.code.clone(),
        label: item.label.clone(),
        source: item.source.clone(),
        priority: item.priority.to_string(),
        notes: text::truncate(&item.notes, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_action_item_draft_views(items: &[DefaultActionItemDraft]) -> Vec<DefaultActionItemDraftView> {
    let mut views: Vec<DefaultActionItemDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_action_item_draft_view(item));
    }
    views
}

pub fn sum_default_action_item_draft_view(view: &DefaultActionItemDraftView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_action_item_draft("code-778", "label-740");
        let view = to_default_action_item_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_action_item_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
