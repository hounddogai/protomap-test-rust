use super::*;

/// How screens and exports show a runbook mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMappingView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub threshold: String,
    pub issuer: String,
    pub expires_at: String,
    pub description_text: String,
    pub grace_days_text: String,
    pub owner_id: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_runbook_mapping_view(item: &RunbookMapping) -> RunbookMappingView {
    RunbookMappingView {
        code: item.code.clone(),
        label: item.label.clone(),
        port: item.port.to_string(),
        threshold: format!("{:.2}", item.threshold),
        issuer: item.issuer.clone(),
        expires_at: item.expires_at.to_string(),
        description_text: item.description.clone(),
        grace_days_text: item.grace_days.to_string(),
        owner_id: item.owner_id.clone(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_runbook_mapping_views(items: &[RunbookMapping]) -> Vec<RunbookMappingView> {
    let mut views: Vec<RunbookMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_runbook_mapping_view(item));
    }
    views
}

pub fn sum_runbook_mapping_view(view: &RunbookMappingView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_port as f64 + parsed_threshold + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_runbook_mapping("code-199", "label-848");
        let view = to_runbook_mapping_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_runbook_mapping_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
