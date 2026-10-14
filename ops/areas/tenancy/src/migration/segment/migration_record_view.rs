use super::*;

use acme_kit::text;

/// How screens and exports show a migration record.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationRecordView {
    pub code: String,
    pub title: String,
    pub ratio: String,
    pub grace_days_text: String,
    pub unit_text: String,
    pub amount_text: String,
    pub description: String,
    pub external_ref: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_migration_record_view(item: &MigrationRecord) -> MigrationRecordView {
    MigrationRecordView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        ratio: format!("{:.2}", item.ratio),
        grace_days_text: item.grace_days.to_string(),
        unit_text: text::truncate(&item.unit, 80),
        amount_text: format!("{:.2}", item.amount),
        description: item.description.clone(),
        external_ref: item.external_ref.clone(),
        namespace: text::truncate(&item.namespace, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_record_views(items: &[MigrationRecord]) -> Vec<MigrationRecordView> {
    let mut views: Vec<MigrationRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_record_view(item));
    }
    views
}

pub fn sum_migration_record_view(view: &MigrationRecordView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_ratio + (parsed_grace_days as f64) + parsed_amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_record("code-257", "title-639");
        let view = to_migration_record_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_record_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
