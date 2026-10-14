use super::*;

/// How screens and exports show a policy allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicyAllocationView {
    pub code_text: String,
    pub name_text: String,
    pub batch_size: String,
    pub minimum: String,
    pub port_text: String,
    pub category_text: String,
    pub cost: String,
    pub currency_text: String,
    pub status_label: String,
}

pub fn to_policy_allocation_view(item: &PolicyAllocation) -> PolicyAllocationView {
    PolicyAllocationView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        minimum: format!("{:.2}", item.minimum),
        port_text: item.port.to_string(),
        category_text: item.category.clone(),
        cost: format!("{:.2}", item.cost),
        currency_text: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_policy_allocation_views(items: &[PolicyAllocation]) -> Vec<PolicyAllocationView> {
    let mut views: Vec<PolicyAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_policy_allocation_view(item));
    }
    views
}

pub fn sum_policy_allocation_view(view: &PolicyAllocationView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_minimum + (parsed_port as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_policy_allocation("code-633", "name-199");
        let view = to_policy_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_policy_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
