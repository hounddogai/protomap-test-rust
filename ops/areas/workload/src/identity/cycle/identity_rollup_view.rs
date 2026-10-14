use super::*;

use acme_kit::text;

/// How screens and exports show a identity rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityRollupView {
    pub code: String,
    pub name: String,
    pub amount: String,
    pub version: String,
    pub category: String,
    pub unit: String,
    pub tenant_id: String,
    pub region_text: String,
    pub weight: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_identity_rollup_view(item: &IdentityRollup) -> IdentityRollupView {
    IdentityRollupView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        amount: format!("{:.2}", item.amount),
        version: item.version.to_string(),
        category: item.category.clone(),
        unit: item.unit.clone(),
        tenant_id: text::truncate(&item.tenant_id, 80),
        region_text: item.region.clone(),
        weight: format!("{:.2}", item.weight),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_rollup_views(items: &[IdentityRollup]) -> Vec<IdentityRollupView> {
    let mut views: Vec<IdentityRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_rollup_view(item));
    }
    views
}

pub fn sum_identity_rollup_view(view: &IdentityRollupView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_amount + (parsed_version as f64) + parsed_weight)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_identity_rollup("code-429", "name-685");
        let view = to_identity_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_identity_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
