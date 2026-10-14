use super::*;

use acme_kit::text;

/// How screens and exports show a migration window.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationWindowView {
    pub code: String,
    pub label: String,
    pub discount: String,
    pub country: String,
    pub owner_id: String,
    pub retries_text: String,
    pub key_id: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_migration_window_view(item: &MigrationWindow) -> MigrationWindowView {
    MigrationWindowView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        discount: format!("{:.2}", item.discount),
        country: text::truncate(&item.country, 120),
        owner_id: item.owner_id.clone(),
        retries_text: item.retries.to_string(),
        key_id: text::truncate(&item.key_id, 64),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_window_views(items: &[MigrationWindow]) -> Vec<MigrationWindowView> {
    let mut views: Vec<MigrationWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_window_view(item));
    }
    views
}

pub fn sum_migration_window_view(view: &MigrationWindowView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_discount + (parsed_retries as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_window("code-280", "label-454");
        let view = to_migration_window_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_window_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
