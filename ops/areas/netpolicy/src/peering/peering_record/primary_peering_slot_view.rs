use super::*;

use acme_kit::text;

/// How screens and exports show a primary peering slot.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringSlotView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub ends_at: String,
    pub threshold: String,
    pub label_text: String,
    pub channel: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_primary_peering_slot_view(item: &PrimaryPeeringSlot) -> PrimaryPeeringSlotView {
    PrimaryPeeringSlotView {
        code: text::truncate(&item.code, 40),
        title: text::truncate(&item.title, 80),
        country: item.country.clone(),
        ends_at: item.ends_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        label_text: text::truncate(&item.label, 120),
        channel: item.channel.clone(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_peering_slot_views(items: &[PrimaryPeeringSlot]) -> Vec<PrimaryPeeringSlotView> {
    let mut views: Vec<PrimaryPeeringSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_peering_slot_view(item));
    }
    views
}

pub fn sum_primary_peering_slot_view(view: &PrimaryPeeringSlotView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_threshold + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_peering_slot("code-662", "title-780");
        let view = to_primary_peering_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_peering_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
