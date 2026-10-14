use super::*;

use acme_kit::text;

/// How screens and exports show a global migration rule.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMigrationRuleView {
    pub code: String,
    pub label_text: String,
    pub description: String,
    pub audience: String,
    pub ttl_seconds: String,
    pub external_ref: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_global_migration_rule_view(item: &GlobalMigrationRule) -> GlobalMigrationRuleView {
    GlobalMigrationRuleView {
        code: item.code.clone(),
        label_text: text::truncate(&item.label, 40),
        description: item.description.clone(),
        audience: item.audience.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        external_ref: text::truncate(&item.external_ref, 120),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_migration_rule_views(items: &[GlobalMigrationRule]) -> Vec<GlobalMigrationRuleView> {
    let mut views: Vec<GlobalMigrationRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_migration_rule_view(item));
    }
    views
}

pub fn sum_global_migration_rule_view(view: &GlobalMigrationRuleView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_global_migration_rule("code-302", "label-85");
        let view = to_global_migration_rule_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_global_migration_rule_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
