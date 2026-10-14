use super::*;

use acme_kit::text;

/// How screens and exports show a weighted action item draft.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemDraftView {
    pub code: String,
    pub name: String,
    pub sequence_text: String,
    pub active_text: String,
    pub priority: String,
    pub created_at_text: String,
    pub ends_at_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_weighted_action_item_draft_view(item: &WeightedActionItemDraft) -> WeightedActionItemDraftView {
    WeightedActionItemDraftView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        sequence_text: item.sequence.to_string(),
        active_text: format!("{}", item.active),
        priority: item.priority.to_string(),
        created_at_text: item.created_at.to_string(),
        ends_at_text: item.ends_at.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_action_item_draft_views(items: &[WeightedActionItemDraft]) -> Vec<WeightedActionItemDraftView> {
    let mut views: Vec<WeightedActionItemDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_action_item_draft_view(item));
    }
    views
}

pub fn sum_weighted_action_item_draft_view(view: &WeightedActionItemDraftView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_priority as f64) + (parsed_created_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_action_item_draft("code-647", "name-403");
        let view = to_weighted_action_item_draft_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_action_item_draft_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
