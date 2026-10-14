use super::*;

use acme_kit::text;

/// How screens and exports show a flow log group.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogGroupView {
    pub code_text: String,
    pub title: String,
    pub reviewer_text: String,
    pub floor_value_text: String,
    pub amount: String,
    pub cost_text: String,
    pub status_label: String,
}

pub fn to_flow_log_group_view(item: &FlowLogGroup) -> FlowLogGroupView {
    FlowLogGroupView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        reviewer_text: item.reviewer.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        amount: format!("{:.2}", item.amount),
        cost_text: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_group_views(items: &[FlowLogGroup]) -> Vec<FlowLogGroupView> {
    let mut views: Vec<FlowLogGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_group_view(item));
    }
    views
}

pub fn sum_flow_log_group_view(view: &FlowLogGroupView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_floor_value + parsed_amount + parsed_cost)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_flow_log_group("code-561", "title-872");
        let view = to_flow_log_group_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_flow_log_group_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
