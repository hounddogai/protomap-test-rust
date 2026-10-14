use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive release slot.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseSlotView {
    pub code_text: String,
    pub label_text: String,
    pub pinned_text: String,
    pub discount: String,
    pub owner_id: String,
    pub ratio_text: String,
    pub rate: String,
    pub channel: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_adaptive_release_slot_view(item: &AdaptiveReleaseSlot) -> AdaptiveReleaseSlotView {
    AdaptiveReleaseSlotView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        pinned_text: format!("{}", item.pinned),
        discount: format!("{:.2}", item.discount),
        owner_id: text::truncate(&item.owner_id, 120),
        ratio_text: format!("{:.2}", item.ratio),
        rate: format!("{:.2}", item.rate),
        channel: text::truncate(&item.channel, 64),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_release_slot_views(items: &[AdaptiveReleaseSlot]) -> Vec<AdaptiveReleaseSlotView> {
    let mut views: Vec<AdaptiveReleaseSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_release_slot_view(item));
    }
    views
}

pub fn sum_adaptive_release_slot_view(view: &AdaptiveReleaseSlotView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_discount + parsed_ratio + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_adaptive_release_slot("code-166", "label-15");
        let view = to_adaptive_release_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_adaptive_release_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
