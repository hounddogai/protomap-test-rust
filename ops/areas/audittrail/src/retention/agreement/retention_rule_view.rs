use super::*;

use acme_kit::text;

/// How screens and exports show a retention rule.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRuleView {
    pub code: String,
    pub name: String,
    pub discount: String,
    pub summary: String,
    pub tenant_id: String,
    pub approved: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_retention_rule_view(item: &RetentionRule) -> RetentionRuleView {
    RetentionRuleView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 120),
        discount: format!("{:.2}", item.discount),
        summary: item.summary.clone(),
        tenant_id: text::truncate(&item.tenant_id, 40),
        approved: format!("{}", item.approved),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_rule_views(items: &[RetentionRule]) -> Vec<RetentionRuleView> {
    let mut views: Vec<RetentionRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_rule_view(item));
    }
    views
}

pub fn sum_retention_rule_view(view: &RetentionRuleView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_retention_rule("code-85", "name-848");
        let view = to_retention_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_retention_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
