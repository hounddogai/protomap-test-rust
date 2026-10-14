use super::*;

use acme_kit::text;

/// How screens and exports show a tiered delegation baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationBaselineView {
    pub code: String,
    pub name: String,
    pub starts_at: String,
    pub maximum: String,
    pub issuer: String,
    pub quantity: String,
    pub summary_text: String,
    pub category: String,
    pub ceiling_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_tiered_delegation_baseline_view(item: &TieredDelegationBaseline) -> TieredDelegationBaselineView {
    TieredDelegationBaselineView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        starts_at: item.starts_at.to_string(),
        maximum: format!("{:.2}", item.maximum),
        issuer: item.issuer.clone(),
        quantity: item.quantity.to_string(),
        summary_text: item.summary.clone(),
        category: item.category.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_delegation_baseline_views(items: &[TieredDelegationBaseline]) -> Vec<TieredDelegationBaselineView> {
    let mut views: Vec<TieredDelegationBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_delegation_baseline_view(item));
    }
    views
}

pub fn sum_tiered_delegation_baseline_view(view: &TieredDelegationBaselineView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_maximum + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_delegation_baseline("code-951", "name-805");
        let view = to_tiered_delegation_baseline_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_delegation_baseline_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
