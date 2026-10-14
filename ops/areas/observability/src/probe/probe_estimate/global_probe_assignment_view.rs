use super::*;

/// How screens and exports show a global probe assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeAssignmentView {
    pub code_text: String,
    pub label: String,
    pub rate: String,
    pub audience: String,
    pub port: String,
    pub currency_text: String,
    pub quantity_text: String,
    pub ceiling_text: String,
    pub status_label: String,
}

pub fn to_global_probe_assignment_view(item: &GlobalProbeAssignment) -> GlobalProbeAssignmentView {
    GlobalProbeAssignmentView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        audience: item.audience.clone(),
        port: item.port.to_string(),
        currency_text: item.currency.clone(),
        quantity_text: item.quantity.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_probe_assignment_views(items: &[GlobalProbeAssignment]) -> Vec<GlobalProbeAssignmentView> {
    let mut views: Vec<GlobalProbeAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_probe_assignment_view(item));
    }
    views
}

pub fn sum_global_probe_assignment_view(view: &GlobalProbeAssignmentView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_rate + (parsed_port as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_probe_assignment("code-32", "label-967");
        let view = to_global_probe_assignment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_probe_assignment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
