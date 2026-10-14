use super::*;

use acme_kit::text;

/// How screens and exports show a fallback action item override.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemOverrideView {
    pub code: String,
    pub title_text: String,
    pub rate: String,
    pub namespace: String,
    pub sequence_text: String,
    pub ends_at_text: String,
    pub locale: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_fallback_action_item_override_view(item: &FallbackActionItemOverride) -> FallbackActionItemOverrideView {
    FallbackActionItemOverrideView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        namespace: item.namespace.clone(),
        sequence_text: item.sequence.to_string(),
        ends_at_text: item.ends_at.to_string(),
        locale: text::truncate(&item.locale, 80),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_action_item_override_views(
    items: &[FallbackActionItemOverride],
) -> Vec<FallbackActionItemOverrideView> {
    let mut views: Vec<FallbackActionItemOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_action_item_override_view(item));
    }
    views
}

pub fn sum_fallback_action_item_override_view(view: &FallbackActionItemOverrideView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_rate + (parsed_sequence as f64) + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_action_item_override("code-696", "title-316");
        let view = to_fallback_action_item_override_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_action_item_override_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
