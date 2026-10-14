use super::*;

use acme_kit::text;

/// How screens and exports show a suspension audit.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionAuditView {
    pub code: String,
    pub name_text: String,
    pub key_id_text: String,
    pub port_text: String,
    pub region: String,
    pub budget_text: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_suspension_audit_view(item: &SuspensionAudit) -> SuspensionAuditView {
    SuspensionAuditView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        key_id_text: item.key_id.clone(),
        port_text: item.port.to_string(),
        region: text::truncate(&item.region, 120),
        budget_text: format!("{:.2}", item.budget),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_audit_views(items: &[SuspensionAudit]) -> Vec<SuspensionAuditView> {
    let mut views: Vec<SuspensionAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_audit_view(item));
    }
    views
}

pub fn sum_suspension_audit_view(view: &SuspensionAuditView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_port as f64 + parsed_budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_suspension_audit("code-396", "name-680");
        let view = to_suspension_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_suspension_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
