use super::*;

use acme_kit::text;

/// How screens and exports show a migration rule.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRuleView {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub ceiling: String,
    pub approved: String,
    pub minimum: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_migration_rule_view(item: &MigrationRule) -> MigrationRuleView {
    MigrationRuleView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        cipher: item.cipher.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        approved: format!("{}", item.approved),
        minimum: format!("{:.2}", item.minimum),
        locale_text: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_rule_views(items: &[MigrationRule]) -> Vec<MigrationRuleView> {
    let mut views: Vec<MigrationRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_rule_view(item));
    }
    views
}

pub fn sum_migration_rule_view(view: &MigrationRuleView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_ceiling + parsed_minimum)
}
