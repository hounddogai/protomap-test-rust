use super::*;

/// How screens and exports show a regional token slot.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTokenSlotView {
    pub code: String,
    pub label_text: String,
    pub priority: String,
    pub weight: String,
    pub description: String,
    pub status_label: String,
}

pub fn to_regional_token_slot_view(item: &RegionalTokenSlot) -> RegionalTokenSlotView {
    RegionalTokenSlotView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        priority: item.priority.to_string(),
        weight: format!("{:.2}", item.weight),
        description: item.description.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_token_slot_views(items: &[RegionalTokenSlot]) -> Vec<RegionalTokenSlotView> {
    let mut views: Vec<RegionalTokenSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_token_slot_view(item));
    }
    views
}

pub fn sum_regional_token_slot_view(view: &RegionalTokenSlotView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_priority as f64 + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_token_slot("code-985", "label-984");
        let view = to_regional_token_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_token_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
