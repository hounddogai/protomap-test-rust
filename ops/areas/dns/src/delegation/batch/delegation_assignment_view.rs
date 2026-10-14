use super::*;

/// How screens and exports show a delegation assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationAssignmentView {
    pub code: String,
    pub name: String,
    pub locale: String,
    pub score: String,
    pub key_id: String,
    pub port_text: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_delegation_assignment_view(item: &DelegationAssignment) -> DelegationAssignmentView {
    DelegationAssignmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        locale: item.locale.clone(),
        score: format!("{:.2}", item.score),
        key_id: item.key_id.clone(),
        port_text: item.port.to_string(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_assignment_views(items: &[DelegationAssignment]) -> Vec<DelegationAssignmentView> {
    let mut views: Vec<DelegationAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_assignment_view(item));
    }
    views
}

pub fn sum_delegation_assignment_view(view: &DelegationAssignmentView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_score + (parsed_port as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_assignment("code-480", "name-390");
        let view = to_delegation_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
