use super::*;

use acme_kit::text;

/// How screens and exports show a rollout slot.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSlotView {
    pub code_text: String,
    pub title_text: String,
    pub archived_text: String,
    pub ends_at_text: String,
    pub cipher: String,
    pub description_text: String,
    pub segment: String,
    pub discount_text: String,
    pub score: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_rollout_slot_view(item: &RolloutSlot) -> RolloutSlotView {
    RolloutSlotView {
        code_text: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        archived_text: format!("{}", item.archived),
        ends_at_text: item.ends_at.to_string(),
        cipher: item.cipher.clone(),
        description_text: item.description.clone(),
        segment: text::truncate(&item.segment, 40),
        discount_text: format!("{:.2}", item.discount),
        score: format!("{:.2}", item.score),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollout_slot_views(items: &[RolloutSlot]) -> Vec<RolloutSlotView> {
    let mut views: Vec<RolloutSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollout_slot_view(item));
    }
    views
}

pub fn sum_rollout_slot_view(view: &RolloutSlotView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_discount + parsed_score)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rollout_slot("code-692", "title-598");
        let view = to_rollout_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rollout_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
