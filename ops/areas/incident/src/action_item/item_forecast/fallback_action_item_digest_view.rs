use super::*;

use acme_kit::text;

/// How screens and exports show a fallback action item digest.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemDigestView {
    pub code: String,
    pub label_text: String,
    pub ceiling: String,
    pub rotation_days_text: String,
    pub notes: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_fallback_action_item_digest_view(item: &FallbackActionItemDigest) -> FallbackActionItemDigestView {
    FallbackActionItemDigestView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        rotation_days_text: item.rotation_days.to_string(),
        notes: text::truncate(&item.notes, 120),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_action_item_digest_views(items: &[FallbackActionItemDigest]) -> Vec<FallbackActionItemDigestView> {
    let mut views: Vec<FallbackActionItemDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_action_item_digest_view(item));
    }
    views
}

pub fn sum_fallback_action_item_digest_view(view: &FallbackActionItemDigestView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_ceiling + (parsed_rotation_days as f64) + (parsed_grace_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_action_item_digest("code-735", "label-239");
        let view = to_fallback_action_item_digest_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_action_item_digest_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
