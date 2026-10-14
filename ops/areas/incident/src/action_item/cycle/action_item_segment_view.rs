use super::*;

use acme_kit::text;

/// How screens and exports show a action item segment.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemSegmentView {
    pub code: String,
    pub name_text: String,
    pub cluster: String,
    pub summary_text: String,
    pub issuer: String,
    pub label: String,
    pub principal: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_action_item_segment_view(item: &ActionItemSegment) -> ActionItemSegmentView {
    ActionItemSegmentView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        cluster: text::truncate(&item.cluster, 40),
        summary_text: text::truncate(&item.summary, 40),
        issuer: item.issuer.clone(),
        label: item.label.clone(),
        principal: item.principal.clone(),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_segment_views(items: &[ActionItemSegment]) -> Vec<ActionItemSegmentView> {
    let mut views: Vec<ActionItemSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_segment_view(item));
    }
    views
}

pub fn sum_action_item_segment_view(view: &ActionItemSegmentView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_segment("code-444", "name-419");
        let view = to_action_item_segment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_segment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
