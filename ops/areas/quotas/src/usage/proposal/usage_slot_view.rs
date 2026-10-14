use super::*;

/// How screens and exports show a usage slot.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSlotView {
    pub code: String,
    pub title_text: String,
    pub quantity_text: String,
    pub discount: String,
    pub ttl_seconds_text: String,
    pub maximum_text: String,
    pub minimum_text: String,
    pub status_label: String,
}

pub fn to_usage_slot_view(item: &UsageSlot) -> UsageSlotView {
    UsageSlotView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        quantity_text: item.quantity.to_string(),
        discount: format!("{:.2}", item.discount),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        minimum_text: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_usage_slot_views(items: &[UsageSlot]) -> Vec<UsageSlotView> {
    let mut views: Vec<UsageSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_usage_slot_view(item));
    }
    views
}

pub fn sum_usage_slot_view(view: &UsageSlotView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_discount + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_usage_slot("code-873", "title-948");
        let view = to_usage_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_usage_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
