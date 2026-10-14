use super::*;

/// How screens and exports show a action item audit.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAuditView {
    pub code: String,
    pub title_text: String,
    pub batch_size: String,
    pub name: String,
    pub locale: String,
    pub rotation_days_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_action_item_audit_view(item: &ActionItemAudit) -> ActionItemAuditView {
    ActionItemAuditView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        batch_size: item.batch_size.to_string(),
        name: item.name.clone(),
        locale: item.locale.clone(),
        rotation_days_text: item.rotation_days.to_string(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_audit_views(items: &[ActionItemAudit]) -> Vec<ActionItemAuditView> {
    let mut views: Vec<ActionItemAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_audit_view(item));
    }
    views
}

pub fn sum_action_item_audit_view(view: &ActionItemAuditView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_rotation_days as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_action_item_audit("code-145", "title-968");
        let view = to_action_item_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_action_item_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
