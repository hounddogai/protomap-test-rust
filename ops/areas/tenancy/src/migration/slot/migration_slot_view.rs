use super::*;

use acme_kit::text;

/// How screens and exports show a migration slot.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationSlotView {
    pub code: String,
    pub title: String,
    pub created_at: String,
    pub active: String,
    pub minimum: String,
    pub weight_text: String,
    pub timezone: String,
    pub currency: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_migration_slot_view(item: &MigrationSlot) -> MigrationSlotView {
    MigrationSlotView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 40),
        created_at: item.created_at.to_string(),
        active: format!("{}", item.active),
        minimum: format!("{:.2}", item.minimum),
        weight_text: format!("{:.2}", item.weight),
        timezone: item.timezone.clone(),
        currency: text::truncate(&item.currency, 120),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_slot_views(items: &[MigrationSlot]) -> Vec<MigrationSlotView> {
    let mut views: Vec<MigrationSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_slot_view(item));
    }
    views
}

pub fn sum_migration_slot_view(view: &MigrationSlotView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_minimum + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_slot("code-417", "title-744");
        let view = to_migration_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
