use super::*;

use acme_kit::text;

/// How screens and exports show a limit rule.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitRuleView {
    pub code_text: String,
    pub title: String,
    pub country_text: String,
    pub revision_text: String,
    pub maximum: String,
    pub key_id_text: String,
    pub pinned: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_limit_rule_view(item: &LimitRule) -> LimitRuleView {
    LimitRuleView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        country_text: item.country.clone(),
        revision_text: item.revision.to_string(),
        maximum: format!("{:.2}", item.maximum),
        key_id_text: item.key_id.clone(),
        pinned: format!("{}", item.pinned),
        checksum_text: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_limit_rule_views(items: &[LimitRule]) -> Vec<LimitRuleView> {
    let mut views: Vec<LimitRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_limit_rule_view(item));
    }
    views
}

pub fn sum_limit_rule_view(view: &LimitRuleView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_revision as f64 + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_limit_rule("code-175", "title-493");
        let view = to_limit_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_limit_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
