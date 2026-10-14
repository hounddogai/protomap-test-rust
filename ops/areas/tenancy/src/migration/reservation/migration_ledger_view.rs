use super::*;

use acme_kit::text;

/// How screens and exports show a migration ledger.
#[derive(Debug, Clone, PartialEq)]
pub struct MigrationLedgerView {
    pub code: String,
    pub label: String,
    pub active: String,
    pub budget_text: String,
    pub country: String,
    pub audience: String,
    pub title: String,
    pub discount_text: String,
    pub cipher: String,
    pub unit: String,
    pub floor_value: String,
    pub status_label: String,
}

pub fn to_migration_ledger_view(item: &MigrationLedger) -> MigrationLedgerView {
    MigrationLedgerView {
        code: item.code.clone(),
        label: item.label.clone(),
        active: format!("{}", item.active),
        budget_text: format!("{:.2}", item.budget),
        country: item.country.clone(),
        audience: text::truncate(&item.audience, 64),
        title: item.title.clone(),
        discount_text: format!("{:.2}", item.discount),
        cipher: item.cipher.clone(),
        unit: item.unit.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_migration_ledger_views(items: &[MigrationLedger]) -> Vec<MigrationLedgerView> {
    let mut views: Vec<MigrationLedgerView> = Vec::new();
    for item in items.iter() {
        views.push(to_migration_ledger_view(item));
    }
    views
}

pub fn sum_migration_ledger_view(view: &MigrationLedgerView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_budget + parsed_discount + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_migration_ledger("code-914", "label-707");
        let view = to_migration_ledger_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_migration_ledger_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
