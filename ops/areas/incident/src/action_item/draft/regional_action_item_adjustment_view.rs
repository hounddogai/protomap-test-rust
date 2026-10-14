use super::*;

/// How screens and exports show a regional action item adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemAdjustmentView {
    pub code: String,
    pub name_text: String,
    pub floor_value: String,
    pub weight: String,
    pub sequence: String,
    pub cipher: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_regional_action_item_adjustment_view(
    item: &RegionalActionItemAdjustment,
) -> RegionalActionItemAdjustmentView {
    RegionalActionItemAdjustmentView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        weight: format!("{:.2}", item.weight),
        sequence: item.sequence.to_string(),
        cipher: item.cipher.clone(),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_action_item_adjustment_views(
    items: &[RegionalActionItemAdjustment],
) -> Vec<RegionalActionItemAdjustmentView> {
    let mut views: Vec<RegionalActionItemAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_action_item_adjustment_view(item));
    }
    views
}

pub fn sum_regional_action_item_adjustment_view(view: &RegionalActionItemAdjustmentView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_floor_value + parsed_weight + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_action_item_adjustment("code-823", "name-926");
        let view = to_regional_action_item_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_action_item_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
