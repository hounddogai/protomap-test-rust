use super::*;

use acme_kit::text;

/// How screens and exports show a capped rollout audit.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRolloutAuditView {
    pub code: String,
    pub name_text: String,
    pub port_text: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_capped_rollout_audit_view(item: &CappedRolloutAudit) -> CappedRolloutAuditView {
    CappedRolloutAuditView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 120),
        port_text: item.port.to_string(),
        audience: text::truncate(&item.audience, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_rollout_audit_views(items: &[CappedRolloutAudit]) -> Vec<CappedRolloutAuditView> {
    let mut views: Vec<CappedRolloutAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_rollout_audit_view(item));
    }
    views
}

pub fn sum_capped_rollout_audit_view(view: &CappedRolloutAuditView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_port as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_rollout_audit("code-923", "name-794");
        let view = to_capped_rollout_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_rollout_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
