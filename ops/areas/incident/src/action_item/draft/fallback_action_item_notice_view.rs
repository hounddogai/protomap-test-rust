use super::*;

use acme_kit::text;

/// How screens and exports show a fallback action item notice.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemNoticeView {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub unit: String,
    pub currency: String,
    pub active: String,
    pub priority_text: String,
    pub minimum_text: String,
    pub limit: String,
    pub ends_at_text: String,
    pub fingerprint: String,
    pub status_label: String,
}

pub fn to_fallback_action_item_notice_view(item: &FallbackActionItemNotice) -> FallbackActionItemNoticeView {
    FallbackActionItemNoticeView {
        code: item.code.clone(),
        name: item.name.clone(),
        locale: item.locale.clone(),
        unit: text::truncate(&item.unit, 80),
        currency: item.currency.clone(),
        active: format!("{}", item.active),
        priority_text: item.priority.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        limit: item.limit.to_string(),
        ends_at_text: item.ends_at.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_action_item_notice_views(items: &[FallbackActionItemNotice]) -> Vec<FallbackActionItemNoticeView> {
    let mut views: Vec<FallbackActionItemNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_action_item_notice_view(item));
    }
    views
}

pub fn sum_fallback_action_item_notice_view(view: &FallbackActionItemNoticeView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_priority as f64 + parsed_minimum + (parsed_limit as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_action_item_notice("code-607", "name-857");
        let view = to_fallback_action_item_notice_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_action_item_notice_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
