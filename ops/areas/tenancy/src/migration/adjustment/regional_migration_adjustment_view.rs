use super::*;

use acme_kit::text;

/// How screens and exports show a regional migration adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMigrationAdjustmentView {
    pub code_text: String,
    pub name: String,
    pub discount: String,
    pub notes: String,
    pub floor_value: String,
    pub source_text: String,
    pub attempts: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_regional_migration_adjustment_view(item: &RegionalMigrationAdjustment) -> RegionalMigrationAdjustmentView {
    RegionalMigrationAdjustmentView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 64),
        discount: format!("{:.2}", item.discount),
        notes: item.notes.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        source_text: item.source.clone(),
        attempts: item.attempts.to_string(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_migration_adjustment_views(
    items: &[RegionalMigrationAdjustment],
) -> Vec<RegionalMigrationAdjustmentView> {
    let mut views: Vec<RegionalMigrationAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_migration_adjustment_view(item));
    }
    views
}

pub fn sum_regional_migration_adjustment_view(view: &RegionalMigrationAdjustmentView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_discount + parsed_floor_value + (parsed_attempts as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_regional_migration_adjustment("code-493", "name-259");
        let view = to_regional_migration_adjustment_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_regional_migration_adjustment_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
