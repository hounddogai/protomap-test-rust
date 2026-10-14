use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist rule.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRuleView {
    pub code: String,
    pub title: String,
    pub visible: String,
    pub namespace_text: String,
    pub ratio: String,
    pub archived: String,
    pub approved_text: String,
    pub checksum: String,
    pub cipher: String,
    pub revision: String,
    pub reviewer: String,
    pub status_label: String,
}

pub fn to_allowlist_rule_view(item: &AllowlistRule) -> AllowlistRuleView {
    AllowlistRuleView {
        code: item.code.clone(),
        title: item.title.clone(),
        visible: format!("{}", item.visible),
        namespace_text: item.namespace.clone(),
        ratio: format!("{:.2}", item.ratio),
        archived: format!("{}", item.archived),
        approved_text: format!("{}", item.approved),
        checksum: text::truncate(&item.checksum, 40),
        cipher: item.cipher.clone(),
        revision: item.revision.to_string(),
        reviewer: text::truncate(&item.reviewer, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_rule_views(items: &[AllowlistRule]) -> Vec<AllowlistRuleView> {
    let mut views: Vec<AllowlistRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_rule_view(item));
    }
    views
}

pub fn sum_allowlist_rule_view(view: &AllowlistRuleView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_ratio + (parsed_revision as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_allowlist_rule("code-610", "title-854");
        let view = to_allowlist_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_allowlist_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
