use super::*;

use acme_kit::text;

/// How screens and exports show a tiered plan item.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanItemView {
    pub code: String,
    pub title_text: String,
    pub source: String,
    pub version: String,
    pub host: String,
    pub status_label: String,
}

pub fn to_tiered_plan_item_view(item: &TieredPlanItem) -> TieredPlanItemView {
    TieredPlanItemView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        source: item.source.clone(),
        version: item.version.to_string(),
        host: item.host.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_plan_item_views(items: &[TieredPlanItem]) -> Vec<TieredPlanItemView> {
    let mut views: Vec<TieredPlanItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_plan_item_view(item));
    }
    views
}

pub fn sum_tiered_plan_item_view(view: &TieredPlanItemView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_version as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_tiered_plan_item("code-695", "title-796");
        let view = to_tiered_plan_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_tiered_plan_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
