use super::*;

use acme_kit::text;

/// How screens and exports show a rolling delegation slot.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDelegationSlotView {
    pub code: String,
    pub name: String,
    pub quantity_text: String,
    pub namespace: String,
    pub segment: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_rolling_delegation_slot_view(item: &RollingDelegationSlot) -> RollingDelegationSlotView {
    RollingDelegationSlotView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        quantity_text: item.quantity.to_string(),
        namespace: text::truncate(&item.namespace, 120),
        segment: item.segment.clone(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_delegation_slot_views(items: &[RollingDelegationSlot]) -> Vec<RollingDelegationSlotView> {
    let mut views: Vec<RollingDelegationSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_delegation_slot_view(item));
    }
    views
}

pub fn sum_rolling_delegation_slot_view(view: &RollingDelegationSlotView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_delegation_slot("code-330", "name-394");
        let view = to_rolling_delegation_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_delegation_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
