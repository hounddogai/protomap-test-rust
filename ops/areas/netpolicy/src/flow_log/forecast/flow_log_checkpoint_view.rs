use super::*;

/// How screens and exports show a flow log checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCheckpointView {
    pub code: String,
    pub name: String,
    pub budget_text: String,
    pub unit_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_flow_log_checkpoint_view(item: &FlowLogCheckpoint) -> FlowLogCheckpointView {
    FlowLogCheckpointView {
        code: item.code.clone(),
        name: item.name.clone(),
        budget_text: format!("{:.2}", item.budget),
        unit_text: item.unit.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_checkpoint_views(items: &[FlowLogCheckpoint]) -> Vec<FlowLogCheckpointView> {
    let mut views: Vec<FlowLogCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_checkpoint_view(item));
    }
    views
}

pub fn sum_flow_log_checkpoint_view(view: &FlowLogCheckpointView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_budget + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_checkpoint("code-209", "name-731");
        let view = to_flow_log_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
