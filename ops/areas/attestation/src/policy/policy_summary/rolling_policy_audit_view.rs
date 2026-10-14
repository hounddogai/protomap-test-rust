use super::*;

use acme_kit::text;

/// How screens and exports show a rolling policy audit.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyAuditView {
    pub code: String,
    pub title: String,
    pub archived: String,
    pub ends_at: String,
    pub margin_text: String,
    pub phone: String,
    pub reviewer: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_rolling_policy_audit_view(item: &RollingPolicyAudit) -> RollingPolicyAuditView {
    RollingPolicyAuditView {
        code: item.code.clone(),
        title: item.title.clone(),
        archived: format!("{}", item.archived),
        ends_at: item.ends_at.to_string(),
        margin_text: format!("{:.2}", item.margin),
        phone: item.phone.clone(),
        reviewer: text::truncate(&item.reviewer, 40),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_policy_audit_views(items: &[RollingPolicyAudit]) -> Vec<RollingPolicyAuditView> {
    let mut views: Vec<RollingPolicyAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_policy_audit_view(item));
    }
    views
}

pub fn sum_rolling_policy_audit_view(view: &RollingPolicyAuditView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_margin + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_rolling_policy_audit("code-651", "title-459");
        let view = to_rolling_policy_audit_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_rolling_policy_audit_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
