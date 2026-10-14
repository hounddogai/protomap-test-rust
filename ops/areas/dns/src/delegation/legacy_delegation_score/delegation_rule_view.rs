use super::*;

use acme_kit::text;

/// How screens and exports show a delegation rule.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRuleView {
    pub code: String,
    pub label: String,
    pub pinned: String,
    pub port: String,
    pub maximum_text: String,
    pub expires_at: String,
    pub ends_at_text: String,
    pub owner_id_text: String,
    pub contact_email_text: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_delegation_rule_view(item: &DelegationRule) -> DelegationRuleView {
    DelegationRuleView {
        code: item.code.clone(),
        label: item.label.clone(),
        pinned: format!("{}", item.pinned),
        port: item.port.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        expires_at: item.expires_at.to_string(),
        ends_at_text: item.ends_at.to_string(),
        owner_id_text: item.owner_id.clone(),
        contact_email_text: item.contact_email.clone(),
        reviewer: text::truncate(&item.reviewer, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_rule_views(items: &[DelegationRule]) -> Vec<DelegationRuleView> {
    let mut views: Vec<DelegationRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_rule_view(item));
    }
    views
}

pub fn sum_delegation_rule_view(view: &DelegationRuleView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_port as f64 + parsed_maximum + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_delegation_rule("code-157", "label-508");
        let view = to_delegation_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_delegation_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
