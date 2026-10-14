use super::*;

use acme_kit::text;

/// How screens and exports show a migration checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationCheckpointView {
    pub code: String,
    pub title: String,
    pub country: String,
    pub cluster: String,
    pub key_id_text: String,
    pub discount: String,
    pub scope: String,
    pub amount: String,
    pub tenant_id: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_migration_checkpoint_view(item: &MigrationCheckpoint) -> MigrationCheckpointView {
    MigrationCheckpointView {
        code: item.code.clone(),
        title: item.title.clone(),
        country: item.country.clone(),
        cluster: item.cluster.clone(),
        key_id_text: item.key_id.clone(),
        discount: format!("{:.2}", item.discount),
        scope: text::truncate(&item.scope, 64),
        amount: format!("{:.2}", item.amount),
        tenant_id: item.tenant_id.clone(),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_checkpoint_views(items: &[MigrationCheckpoint]) -> Vec<MigrationCheckpointView> {
    let mut views: Vec<MigrationCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_checkpoint_view(item));
    }
    views
}

pub fn sum_migration_checkpoint_view(view: &MigrationCheckpointView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_discount + parsed_amount + parsed_maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_checkpoint("code-837", "title-312");
        let view = to_migration_checkpoint_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_checkpoint_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
