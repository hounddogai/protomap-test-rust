use super::*;

use acme_kit::text;

/// How screens and exports show a tiered migration adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationAdjustmentView {
    pub code: String,
    pub label_text: String,
    pub capacity: String,
    pub issuer: String,
    pub score: String,
    pub currency: String,
    pub quantity: String,
    pub timezone: String,
    pub key_id: String,
    pub status_label: String,
}

pub fn to_tiered_migration_adjustment_view(item: &TieredMigrationAdjustment) -> TieredMigrationAdjustmentView {
    TieredMigrationAdjustmentView {
        code: text::truncate(&item.code, 64),
        label_text: item.label.clone(),
        capacity: item.capacity.to_string(),
        issuer: item.issuer.clone(),
        score: format!("{:.2}", item.score),
        currency: text::truncate(&item.currency, 40),
        quantity: item.quantity.to_string(),
        timezone: item.timezone.clone(),
        key_id: item.key_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_migration_adjustment_views(items: &[TieredMigrationAdjustment]) -> Vec<TieredMigrationAdjustmentView> {
    let mut views: Vec<TieredMigrationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_migration_adjustment_view(item));
    }
    views
}

pub fn sum_tiered_migration_adjustment_view(view: &TieredMigrationAdjustmentView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_capacity as f64 + parsed_score + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_migration_adjustment("code-859", "label-261");
        let view = to_tiered_migration_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_migration_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
