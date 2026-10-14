use super::*;

/// How screens and exports show a weighted action item cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedActionItemCycleView {
    pub code_text: String,
    pub name: String,
    pub port: String,
    pub limit: String,
    pub owner_id: String,
    pub notes_text: String,
    pub capacity: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_weighted_action_item_cycle_view(item: &WeightedActionItemCycle) -> WeightedActionItemCycleView {
    WeightedActionItemCycleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        port: item.port.to_string(),
        limit: item.limit.to_string(),
        owner_id: item.owner_id.clone(),
        notes_text: item.notes.clone(),
        capacity: item.capacity.to_string(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_action_item_cycle_views(items: &[WeightedActionItemCycle]) -> Vec<WeightedActionItemCycleView> {
    let mut views: Vec<WeightedActionItemCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_action_item_cycle_view(item));
    }
    views
}

pub fn sum_weighted_action_item_cycle_view(view: &WeightedActionItemCycleView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_port as f64 + (parsed_limit as f64) + (parsed_capacity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_weighted_action_item_cycle("code-93", "name-732");
        let view = to_weighted_action_item_cycle_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_weighted_action_item_cycle_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
