use super::*;

use acme_kit::text;

/// How screens and exports show a legacy action item review.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyActionItemReviewView {
    pub code_text: String,
    pub label: String,
    pub visible_text: String,
    pub namespace: String,
    pub quantity_text: String,
    pub rotation_days_text: String,
    pub maximum: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_legacy_action_item_review_view(item: &LegacyActionItemReview) -> LegacyActionItemReviewView {
    LegacyActionItemReviewView {
        code_text: text::truncate(&item.code, 80),
        label: item.label.clone(),
        visible_text: format!("{}", item.visible),
        namespace: text::truncate(&item.namespace, 120),
        quantity_text: item.quantity.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        maximum: format!("{:.2}", item.maximum),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_action_item_review_views(items: &[LegacyActionItemReview]) -> Vec<LegacyActionItemReviewView> {
    let mut views: Vec<LegacyActionItemReviewView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_action_item_review_view(item));
    }
    views
}

pub fn sum_legacy_action_item_review_view(view: &LegacyActionItemReviewView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_quantity as f64 + (parsed_rotation_days as f64) + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_legacy_action_item_review("code-554", "label-748");
        let view = to_legacy_action_item_review_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_legacy_action_item_review_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
