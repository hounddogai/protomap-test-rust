use super::*;

use acme_kit::text;

/// How screens and exports show a overage rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageRollupView {
    pub code: String,
    pub name_text: String,
    pub category_text: String,
    pub audience: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_overage_rollup_view(item: &OverageRollup) -> OverageRollupView {
    OverageRollupView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 40),
        category_text: item.category.clone(),
        audience: item.audience.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_rollup_views(items: &[OverageRollup]) -> Vec<OverageRollupView> {
    let mut views: Vec<OverageRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_rollup_view(item));
    }
    views
}

pub fn sum_overage_rollup_view(view: &OverageRollupView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_overage_rollup("code-45", "name-683");
        let view = to_overage_rollup_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_overage_rollup_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
