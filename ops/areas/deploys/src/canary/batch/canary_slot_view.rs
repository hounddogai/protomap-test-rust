use super::*;

use acme_kit::text;

/// How screens and exports show a canary slot.
#[derive(Debug, Clone, PartialEq)]
pub struct CanarySlotView {
    pub code: String,
    pub title_text: String,
    pub limit: String,
    pub active: String,
    pub ratio_text: String,
    pub notes_text: String,
    pub currency: String,
    pub amount: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_canary_slot_view(item: &CanarySlot) -> CanarySlotView {
    CanarySlotView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        limit: item.limit.to_string(),
        active: format!("{}", item.active),
        ratio_text: format!("{:.2}", item.ratio),
        notes_text: item.notes.clone(),
        currency: item.currency.clone(),
        amount: format!("{:.2}", item.amount),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_slot_views(items: &[CanarySlot]) -> Vec<CanarySlotView> {
    let mut views: Vec<CanarySlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_slot_view(item));
    }
    views
}

pub fn sum_canary_slot_view(view: &CanarySlotView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_limit as f64 + parsed_ratio + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_canary_slot("code-940", "title-807");
        let view = to_canary_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_canary_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
