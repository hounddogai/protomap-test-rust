use super::*;

/// How screens and exports show a default action item audit.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemAuditView {
    pub code: String,
    pub name_text: String,
    pub description_text: String,
    pub threshold: String,
    pub ends_at_text: String,
    pub rate: String,
    pub retries: String,
    pub batch_size_text: String,
    pub status_label: String,
}

pub fn to_default_action_item_audit_view(item: &DefaultActionItemAudit) -> DefaultActionItemAuditView {
    DefaultActionItemAuditView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        description_text: item.description.clone(),
        threshold: format!("{:.2}", item.threshold),
        ends_at_text: item.ends_at.to_string(),
        rate: format!("{:.2}", item.rate),
        retries: item.retries.to_string(),
        batch_size_text: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_action_item_audit_views(items: &[DefaultActionItemAudit]) -> Vec<DefaultActionItemAuditView> {
    let mut views: Vec<DefaultActionItemAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_action_item_audit_view(item));
    }
    views
}

pub fn sum_default_action_item_audit_view(view: &DefaultActionItemAuditView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ends_at = view
        .ends_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_threshold + (parsed_ends_at as f64) + parsed_rate)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_default_action_item_audit("code-523", "name-90");
        let view = to_default_action_item_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_default_action_item_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
