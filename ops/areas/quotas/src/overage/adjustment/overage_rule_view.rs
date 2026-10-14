use super::*;

use acme_kit::text;

/// How screens and exports show a overage rule.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageRuleView {
    pub code_text: String,
    pub label: String,
    pub version: String,
    pub locale: String,
    pub created_at_text: String,
    pub discount_text: String,
    pub cluster_text: String,
    pub timezone: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_overage_rule_view(item: &OverageRule) -> OverageRuleView {
    OverageRuleView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        version: item.version.to_string(),
        locale: text::truncate(&item.locale, 80),
        created_at_text: item.created_at.to_string(),
        discount_text: format!("{:.2}", item.discount),
        cluster_text: text::truncate(&item.cluster, 80),
        timezone: item.timezone.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_rule_views(items: &[OverageRule]) -> Vec<OverageRuleView> {
    let mut views: Vec<OverageRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_rule_view(item));
    }
    views
}

pub fn sum_overage_rule_view(view: &OverageRuleView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_version as f64 + (parsed_created_at as f64) + parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_rule("code-734", "label-798");
        let view = to_overage_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
