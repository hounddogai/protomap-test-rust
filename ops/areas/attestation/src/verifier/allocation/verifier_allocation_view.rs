use super::*;

/// How screens and exports show a verifier allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAllocationView {
    pub code_text: String,
    pub name: String,
    pub minimum: String,
    pub tenant_id_text: String,
    pub quantity_text: String,
    pub window_days_text: String,
    pub status_label: String,
}

pub fn to_verifier_allocation_view(item: &VerifierAllocation) -> VerifierAllocationView {
    VerifierAllocationView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        tenant_id_text: item.tenant_id.clone(),
        quantity_text: item.quantity.to_string(),
        window_days_text: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_allocation_views(items: &[VerifierAllocation]) -> Vec<VerifierAllocationView> {
    let mut views: Vec<VerifierAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_allocation_view(item));
    }
    views
}

pub fn sum_verifier_allocation_view(view: &VerifierAllocationView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_minimum + (parsed_quantity as f64) + (parsed_window_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_verifier_allocation("code-351", "name-203");
        let view = to_verifier_allocation_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_verifier_allocation_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
