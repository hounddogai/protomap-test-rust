use super::*;

use acme_kit::text;

/// How screens and exports show a batched slo slot.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSloSlotView {
    pub code: String,
    pub title_text: String,
    pub locale: String,
    pub cost: String,
    pub source: String,
    pub scope: String,
    pub label_text: String,
    pub visible: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_batched_slo_slot_view(item: &BatchedSloSlot) -> BatchedSloSlotView {
    BatchedSloSlotView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        locale: item.locale.clone(),
        cost: format!("{:.2}", item.cost),
        source: text::truncate(&item.source, 64),
        scope: item.scope.clone(),
        label_text: text::truncate(&item.label, 40),
        visible: format!("{}", item.visible),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_slo_slot_views(items: &[BatchedSloSlot]) -> Vec<BatchedSloSlotView> {
    let mut views: Vec<BatchedSloSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_slo_slot_view(item));
    }
    views
}

pub fn sum_batched_slo_slot_view(view: &BatchedSloSlotView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_slo_slot("code-819", "title-624");
        let view = to_batched_slo_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_slo_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
