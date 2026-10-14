use super::*;

use acme_kit::text;

/// How screens and exports show a tiered release slot.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseSlotView {
    pub code: String,
    pub name_text: String,
    pub cluster: String,
    pub unit: String,
    pub category_text: String,
    pub pinned: String,
    pub score_text: String,
    pub status_label: String,
}

pub fn to_tiered_release_slot_view(item: &TieredReleaseSlot) -> TieredReleaseSlotView {
    TieredReleaseSlotView {
        code: text::truncate(&item.code, 120),
        name_text: item.name.clone(),
        cluster: item.cluster.clone(),
        unit: item.unit.clone(),
        category_text: item.category.clone(),
        pinned: format!("{}", item.pinned),
        score_text: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_release_slot_views(items: &[TieredReleaseSlot]) -> Vec<TieredReleaseSlotView> {
    let mut views: Vec<TieredReleaseSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_release_slot_view(item));
    }
    views
}

pub fn sum_tiered_release_slot_view(view: &TieredReleaseSlotView) -> Result<f64, String> {
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
        let item = new_tiered_release_slot("code-829", "name-530");
        let view = to_tiered_release_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_release_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
