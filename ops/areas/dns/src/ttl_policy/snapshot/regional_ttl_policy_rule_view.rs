use super::*;

/// How screens and exports show a regional ttl policy rule.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTtlPolicyRuleView {
    pub code: String,
    pub title: String,
    pub host_text: String,
    pub discount: String,
    pub expires_at_text: String,
    pub status_label: String,
}

pub fn to_regional_ttl_policy_rule_view(item: &RegionalTtlPolicyRule) -> RegionalTtlPolicyRuleView {
    RegionalTtlPolicyRuleView {
        code: item.code.clone(),
        title: item.title.clone(),
        host_text: item.host.clone(),
        discount: format!("{:.2}", item.discount),
        expires_at_text: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_ttl_policy_rule_views(items: &[RegionalTtlPolicyRule]) -> Vec<RegionalTtlPolicyRuleView> {
    let mut views: Vec<RegionalTtlPolicyRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_ttl_policy_rule_view(item));
    }
    views
}

pub fn sum_regional_ttl_policy_rule_view(view: &RegionalTtlPolicyRuleView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_discount + (parsed_expires_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_ttl_policy_rule("code-240", "title-800");
        let view = to_regional_ttl_policy_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_ttl_policy_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
