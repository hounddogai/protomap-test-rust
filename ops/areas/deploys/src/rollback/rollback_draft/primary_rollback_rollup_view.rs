use super::*;

use acme_kit::text;

/// How screens and exports show a primary rollback rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRollbackRollupView {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub category: String,
    pub archived: String,
    pub currency: String,
    pub source_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_primary_rollback_rollup_view(item: &PrimaryRollbackRollup) -> PrimaryRollbackRollupView {
    PrimaryRollbackRollupView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 120),
        audience: item.audience.clone(),
        category: text::truncate(&item.category, 40),
        archived: format!("{}", item.archived),
        currency: text::truncate(&item.currency, 120),
        source_text: item.source.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_rollback_rollup_views(items: &[PrimaryRollbackRollup]) -> Vec<PrimaryRollbackRollupView> {
    let mut views: Vec<PrimaryRollbackRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_rollback_rollup_view(item));
    }
    views
}

pub fn sum_primary_rollback_rollup_view(view: &PrimaryRollbackRollupView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_primary_rollback_rollup("code-681", "label-963");
        let view = to_primary_rollback_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_primary_rollback_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
