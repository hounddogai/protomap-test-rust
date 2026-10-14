use super::*;

use acme_kit::text;

/// How screens and exports show a pending ttl policy rule.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyRuleView {
    pub code: String,
    pub name: String,
    pub reviewer_text: String,
    pub pinned: String,
    pub active: String,
    pub cipher_text: String,
    pub expires_at_text: String,
    pub window_days: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_pending_ttl_policy_rule_view(item: &PendingTtlPolicyRule) -> PendingTtlPolicyRuleView {
    PendingTtlPolicyRuleView {
        code: item.code.clone(),
        name: item.name.clone(),
        reviewer_text: text::truncate(&item.reviewer, 64),
        pinned: format!("{}", item.pinned),
        active: format!("{}", item.active),
        cipher_text: item.cipher.clone(),
        expires_at_text: item.expires_at.to_string(),
        window_days: item.window_days.to_string(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_ttl_policy_rule_views(items: &[PendingTtlPolicyRule]) -> Vec<PendingTtlPolicyRuleView> {
    let mut views: Vec<PendingTtlPolicyRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_ttl_policy_rule_view(item));
    }
    views
}

pub fn sum_pending_ttl_policy_rule_view(view: &PendingTtlPolicyRuleView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_window_days as f64) + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_ttl_policy_rule("code-818", "name-732");
        let view = to_pending_ttl_policy_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_ttl_policy_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
