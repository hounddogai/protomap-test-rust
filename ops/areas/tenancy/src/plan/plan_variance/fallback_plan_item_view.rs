use super::*;

use acme_kit::text;

/// How screens and exports show a fallback plan item.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanItemView {
    pub code: String,
    pub title: String,
    pub weight_text: String,
    pub label_text: String,
    pub owner_id_text: String,
    pub created_at_text: String,
    pub visible: String,
    pub notes: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_fallback_plan_item_view(item: &FallbackPlanItem) -> FallbackPlanItemView {
    FallbackPlanItemView {
        code: item.code.clone(),
        title: item.title.clone(),
        weight_text: format!("{:.2}", item.weight),
        label_text: item.label.clone(),
        owner_id_text: text::truncate(&item.owner_id, 120),
        created_at_text: item.created_at.to_string(),
        visible: format!("{}", item.visible),
        notes: item.notes.clone(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_plan_item_views(items: &[FallbackPlanItem]) -> Vec<FallbackPlanItemView> {
    let mut views: Vec<FallbackPlanItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_plan_item_view(item));
    }
    views
}

pub fn sum_fallback_plan_item_view(view: &FallbackPlanItemView) -> Result<f64, String> {
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_weight + (parsed_created_at as f64) + (parsed_quota_units as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_plan_item("code-864", "title-7");
        let view = to_fallback_plan_item_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_plan_item_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
