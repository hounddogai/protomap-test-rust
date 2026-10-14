use super::*;

use acme_kit::text;

/// How screens and exports show a grant rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantRollupView {
    pub code: String,
    pub name: String,
    pub issuer_text: String,
    pub grace_days: String,
    pub quantity_text: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_grant_rollup_view(item: &GrantRollup) -> GrantRollupView {
    GrantRollupView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 64),
        issuer_text: item.issuer.clone(),
        grace_days: item.grace_days.to_string(),
        quantity_text: item.quantity.to_string(),
        locale: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_rollup_views(items: &[GrantRollup]) -> Vec<GrantRollupView> {
    let mut views: Vec<GrantRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_rollup_view(item));
    }
    views
}

pub fn sum_grant_rollup_view(view: &GrantRollupView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_grant_rollup("code-686", "name-571");
        let view = to_grant_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_grant_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
