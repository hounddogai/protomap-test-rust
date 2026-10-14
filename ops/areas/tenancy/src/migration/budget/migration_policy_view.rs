use super::*;

use acme_kit::text;

/// How screens and exports show a migration policy.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationPolicyView {
    pub code: String,
    pub label_text: String,
    pub unit: String,
    pub attempts: String,
    pub host_text: String,
    pub timezone_text: String,
    pub currency_text: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_migration_policy_view(item: &MigrationPolicy) -> MigrationPolicyView {
    MigrationPolicyView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        unit: item.unit.clone(),
        attempts: item.attempts.to_string(),
        host_text: item.host.clone(),
        timezone_text: text::truncate(&item.timezone, 64),
        currency_text: text::truncate(&item.currency, 64),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_policy_views(items: &[MigrationPolicy]) -> Vec<MigrationPolicyView> {
    let mut views: Vec<MigrationPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_policy_view(item));
    }
    views
}

pub fn sum_migration_policy_view(view: &MigrationPolicyView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_attempts as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_policy("code-195", "label-311");
        let view = to_migration_policy_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_policy_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
