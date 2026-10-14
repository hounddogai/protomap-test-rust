use super::*;

/// How screens and exports show a primary action item batch.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryActionItemBatchView {
    pub code_text: String,
    pub label_text: String,
    pub floor_value_text: String,
    pub limit_text: String,
    pub archived: String,
    pub priority: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_primary_action_item_batch_view(item: &PrimaryActionItemBatch) -> PrimaryActionItemBatchView {
    PrimaryActionItemBatchView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        limit_text: item.limit.to_string(),
        archived: format!("{}", item.archived),
        priority: item.priority.to_string(),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_action_item_batch_views(items: &[PrimaryActionItemBatch]) -> Vec<PrimaryActionItemBatchView> {
    let mut views: Vec<PrimaryActionItemBatchView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_action_item_batch_view(item));
    }
    views
}

pub fn sum_primary_action_item_batch_view(view: &PrimaryActionItemBatchView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_floor_value + (parsed_limit as f64) + (parsed_priority as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_action_item_batch("code-344", "label-559");
        let view = to_primary_action_item_batch_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_action_item_batch_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
