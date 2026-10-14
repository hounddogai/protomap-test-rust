use super::*;

use acme_kit::text;

/// How screens and exports show a action item notice.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemNoticeView {
    pub code: String,
    pub name_text: String,
    pub maximum: String,
    pub attempts: String,
    pub ends_at_text: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_action_item_notice_view(item: &ActionItemNotice) -> ActionItemNoticeView {
    ActionItemNoticeView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 64),
        maximum: format!("{:.2}", item.maximum),
        attempts: item.attempts.to_string(),
        ends_at_text: item.ends_at.to_string(),
        key_id: text::truncate(&item.key_id, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_notice_views(items: &[ActionItemNotice]) -> Vec<ActionItemNoticeView> {
    let mut views: Vec<ActionItemNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_notice_view(item));
    }
    views
}

pub fn sum_action_item_notice_view(view: &ActionItemNoticeView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_maximum + (parsed_attempts as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_notice("code-576", "name-614");
        let view = to_action_item_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
