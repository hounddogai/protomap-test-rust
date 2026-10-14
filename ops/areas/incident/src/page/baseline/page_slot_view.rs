use super::*;

use acme_kit::text;

/// How screens and exports show a page slot.
#[derive(Debug, Clone, PartialEq)]
pub struct PageSlotView {
    pub code: String,
    pub label_text: String,
    pub pinned_text: String,
    pub quantity: String,
    pub weight: String,
    pub issuer: String,
    pub region: String,
    pub unit: String,
    pub retries: String,
    pub principal: String,
    pub status_label: String,
}

pub fn to_page_slot_view(item: &PageSlot) -> PageSlotView {
    PageSlotView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        pinned_text: format!("{}", item.pinned),
        quantity: item.quantity.to_string(),
        weight: format!("{:.2}", item.weight),
        issuer: item.issuer.clone(),
        region: text::truncate(&item.region, 40),
        unit: item.unit.clone(),
        retries: item.retries.to_string(),
        principal: text::truncate(&item.principal, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_slot_views(items: &[PageSlot]) -> Vec<PageSlotView> {
    let mut views: Vec<PageSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_slot_view(item));
    }
    views
}

pub fn sum_page_slot_view(view: &PageSlotView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_weight + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_page_slot("code-3", "label-255");
        let view = to_page_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_page_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
