use super::*;

/// How screens and exports show a action item preference.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemPreferenceView {
    pub code: String,
    pub title_text: String,
    pub channel_text: String,
    pub active: String,
    pub starts_at_text: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_action_item_preference_view(item: &ActionItemPreference) -> ActionItemPreferenceView {
    ActionItemPreferenceView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        channel_text: item.channel.clone(),
        active: format!("{}", item.active),
        starts_at_text: item.starts_at.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_preference_views(items: &[ActionItemPreference]) -> Vec<ActionItemPreferenceView> {
    let mut views: Vec<ActionItemPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_preference_view(item));
    }
    views
}

pub fn sum_action_item_preference_view(view: &ActionItemPreferenceView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_starts_at as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_preference("code-572", "title-999");
        let view = to_action_item_preference_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_preference_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
